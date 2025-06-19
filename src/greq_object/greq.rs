use crate::greq_object::greq_content::GreqContent;
use crate::greq_object::greq_footer::GreqFooter;
use crate::greq_object::greq_header::GreqHeader;
use crate::greq_object::greq_http_request::GreqHttpRequest;
use crate::greq_object::greq_response::GreqResponse;
use crate::greq_object::greq_footer_condition::ConditionOperator;
use crate::greq_object::greq_parser::*;
use crate::constants::{DEFAULT_DELIMITER_CHAR, NEW_LINE};
use futures::future::BoxFuture;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::greq_object::traits::enrich_with_trait::EnrichWith;
use regex;
use std::fs;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Greq {
    pub file_contents: String,
    pub sections_delimiter: char,
    pub header: GreqHeader,
    pub content: GreqContent,
    pub footer: GreqFooter,
}

// possible Greq parsing errors
#[derive(Debug, PartialEq)]
pub enum GreqErrorCodes {
    TooFewSections,
    TooManySections,
    SeparatorNotSet,

    ParsingHeaderSectionFailed,
    ParsingContentSectionFailed,
    ParsingFooterSectionFailed,

    ReadGreqFileError,
    HttpError,
    EnrichmentFailed,
}

#[derive(Debug)]
pub struct GreqError {
    pub code: GreqErrorCodes,
    pub message: String
}

impl GreqErrorCodes {
    pub fn error_message(&self) -> &'static str {
        match self {
            GreqErrorCodes::ReadGreqFileError => "Error reading Greq file.",
            _ => "Unrecognized Greq error",
        }
    }
}

impl GreqError {
    pub fn new(code: GreqErrorCodes, message: &str) -> Self {
        Self { code, message: message.to_string() }
    }

    pub fn from_error_code(code: GreqErrorCodes) -> Self {
        let error_message = code.error_message();
        Self::new(code, error_message)
    }
}

impl FromStr for Greq {
    // define a type for the parsing errors
    type Err = GreqError;

    fn from_str(raw_file_contents: &str) -> Result<Self, Self::Err> {
        // initialize a new Greq object with the original file contents
        let mut greq = Greq {
            file_contents: raw_file_contents.to_string(),
            ..Default::default()
        };

        // check if a custom delimiter was defined in the file
        let delimiter_char = extract_delimiter(raw_file_contents).unwrap_or(DEFAULT_DELIMITER_CHAR);
        greq.sections_delimiter = delimiter_char;

        // greq file must have 3 sections. 
        // the header (metadata), content (http raw request), and footer (the evaluation conditions)
        let sections = parse_sections(raw_file_contents, greq.sections_delimiter)
            .map_err(|e| GreqError::from_error_code(e))?;

        print!("parsing header...");
        greq.header = GreqHeader::parse(&sections[0])
            .map_err(|_e| GreqError::from_error_code(GreqErrorCodes::ParsingHeaderSectionFailed))?;
        println!("done");

        print!("parsing content...");
        greq.content = GreqContent::from_str(&sections[1].join(NEW_LINE))
            .map_err(|_e| GreqError::from_error_code(GreqErrorCodes::ParsingContentSectionFailed))?;
        // set the default protocol to https
        greq.content.http_request.is_http = greq.header.is_http.unwrap_or(false);
        println!("done");

        print!("parsing footer...");
        greq.footer = GreqFooter::from_str(&sections[2].join(NEW_LINE))
            .map_err(|_e| GreqError::from_error_code(GreqErrorCodes::ParsingFooterSectionFailed))?;
        println!("done");

        Ok(greq)
    }
}

impl EnrichWith for Greq {
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
    where
        Self: Sized,
    {
        self.header.enrich_with(&object_to_merge.header)?;
        self.content.enrich_with(&object_to_merge.content)?;
        self.footer.enrich_with(&object_to_merge.footer)?;

        Ok(())
    }
}

impl Greq {
    // Add a new boxed version of the `execute` function to handle recursion.
    fn boxed_execute(&self) -> BoxFuture<'_, Result<(Option<bool>, String), GreqError>> {
        Box::pin(self.execute())
    }

    /// execute the request and return the evaluation result and the raw response.
    pub async fn execute(&self) -> Result<(Option<bool>, String), GreqError> {
        if let Some(ref depends_on_request_path) = self.header.depends_on {
            // get_response the dependant request
            let dependant_request = Greq::from_file(depends_on_request_path)?;
            dependant_request.boxed_execute().await?;
        }

        let result = self.get_response().await.map_err(|e| {
            GreqError::new(GreqErrorCodes::HttpError, &e)
        })?;

        let evaluation_result = match self.evaluate().await {
            Ok(res) => Some(res),
            Err(e) => {
                println!("Error evaluating conditions: {}", e);
                return Err(GreqError::new(GreqErrorCodes::HttpError, &e));
            }
        };

        Ok((evaluation_result, result.body.unwrap()))
    }

    pub async fn get_response(&self) -> Result<GreqResponse, String> {
        // Create a reqwest client
        let client = Client::new();

        // Set up the request builder based on the method in `Greq`
        let full_url = self.content.http_request.get_full_url();
        println!(
            "sending request to {}.\r\nself.content.http_request: {:?}\r\n",
            full_url, self.content.http_request
        );
        let request_builder = match self.content.http_request.method.to_lowercase().as_str() {
            "get" => client.get(full_url),
            "post" => client.post(full_url),
            "put" => client.put(full_url),
            "delete" => client.delete(full_url),
            "head" => client.head(full_url),
            "patch" => client.patch(full_url),
            _ => return Err("Unsupported HTTP method".to_string()),
        };

        // Add headers if any
        let mut request_builder = request_builder;
        for (key, value) in &self.content.http_request.headers {
            request_builder = request_builder.header(key, value);
        }

        // Add the body if it's a POST, PUT, or PATCH request
        if !self.content.http_request.content.is_empty() {
            request_builder = request_builder.body(self.content.http_request.content.clone());
        }

        // Execute the request
        let response = request_builder.send().await.map_err(|e| e.to_string())?;

        // Parse the response
        let status_code = response.status().as_u16();
        let reason_phrase = response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .to_string();

        // Collect headers into a HashMap
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        // Get the body (if any)
        let body = response.text().await.map_err(|e| e.to_string())?;

        // Return the GreqResponse
        Ok(GreqResponse {
            status_code,
            reason_phrase,
            headers,
            body: Some(body.clone()),
        })
    }

    // Load a Greq object from a file
    pub fn from_file(file_path: &str) -> Result<Greq, GreqError> {
        let file_contents = fs::read_to_string(file_path).map_err(|e| {
            GreqError::new(GreqErrorCodes::ReadGreqFileError, &format!("Failed to read file {}: {}", file_path, e))
        })?;

        // parse the file contents into a Greq object
        let mut greq = Greq::from_str(&file_contents)?;

        // load the base request if specified
        // "ref" is used to avoid cloning the string unnecessarily
        if let Some(ref base_request_path) = greq.header.base_request {
            let base_greq = Greq::from_file(base_request_path)?;

            // merge the base request into the current request
            greq.enrich_with(&base_greq).map_err(|e| GreqError::new(GreqErrorCodes::EnrichmentFailed, &e))?;
        }

        Ok(greq)
    }

    pub fn get_http_request(&self, base_request: Option<Greq>) -> Result<&GreqHttpRequest, String> {
        // check that base request is provided
        if self.header.base_request.is_some() && !base_request.is_some() {
            return Err("base_request isn't provided".to_string());
        }

        Ok(&self.content.http_request)
    }

    async fn evaluate(&self) -> Result<bool, String> {
        let mut result = true;
        let mut current_or_group = Vec::new();
        let mut current_or_result = false;
        let response = self.get_response().await?;
        let status_code = response.status_code;
        let response_body_owned = response.body.clone().unwrap_or_else(String::new);
        let response_body = &response_body_owned;
        let headers = &response.headers;

        for condition in &self.footer.conditions {
            let condition_result = match condition.key.as_str() {
                "status-code" => {
                    let expected = condition.value.parse::<u16>().map_err(|e| format!("Invalid status code: {}", e))?;
                    match condition.operator {
                        ConditionOperator::Equals => status_code == expected,
                        _ => return Err("Only equals operator is supported for status-code".to_string()),
                    }
                }
                "response-content" => {
                    let expected = condition.value.trim_matches('"');
                    match condition.operator {
                        ConditionOperator::Equals => response_body == expected,
                        ConditionOperator::Contains => {
                            if condition.is_regex {
                                let re = regex::Regex::new(expected).map_err(|e| format!("Invalid regex: {}", e))?;
                                re.is_match(response_body)
                            } else {
                                if condition.is_case_sensitive {
                                    response_body.contains(expected)
                                } else {
                                    response_body.to_lowercase().contains(&expected.to_lowercase())
                                }
                            }
                        }
                        ConditionOperator::StartsWith => {
                            if condition.is_case_sensitive {
                                response_body.starts_with(expected)
                            } else {
                                response_body.to_lowercase().starts_with(&expected.to_lowercase())
                            }
                        }
                        ConditionOperator::EndsWith => {
                            if condition.is_case_sensitive {
                                response_body.ends_with(expected)
                            } else {
                                response_body.to_lowercase().ends_with(&expected.to_lowercase())
                            }
                        }
                    }
                }
                header_name => {
                    let header_value = headers.get(header_name).map(|v| v.as_str()).unwrap_or("");
                    let expected = condition.value.trim_matches('"');
                    match condition.operator {
                        ConditionOperator::Equals => header_value == expected,
                        ConditionOperator::Contains => {
                            if condition.is_regex {
                                let re = regex::Regex::new(expected).map_err(|e| format!("Invalid regex: {}", e))?;
                                re.is_match(header_value)
                            } else {
                                if condition.is_case_sensitive {
                                    header_value.contains(expected)
                                } else {
                                    header_value.to_lowercase().contains(&expected.to_lowercase())
                                }
                            }
                        }
                        ConditionOperator::StartsWith => {
                            if condition.is_case_sensitive {
                                header_value.starts_with(expected)
                            } else {
                                header_value.to_lowercase().starts_with(&expected.to_lowercase())
                            }
                        }
                        ConditionOperator::EndsWith => {
                            if condition.is_case_sensitive {
                                header_value.ends_with(expected)
                            } else {
                                header_value.to_lowercase().ends_with(&expected.to_lowercase())
                            }
                        }
                    }
                }
            };

            let final_result = if condition.has_not {
                !condition_result
            } else {
                condition_result
            };

            if condition.has_or {
                current_or_group.push(final_result);
            } else {
                if !current_or_group.is_empty() {
                    current_or_result = current_or_group.iter().any(|&x| x);
                    current_or_group.clear();
                }
                if !current_or_result {
                    result = false;
                    break;
                }
                result &= final_result;
            }
        }

        // Handle any remaining OR conditions
        if !current_or_group.is_empty() {
            current_or_result = current_or_group.iter().any(|&x| x);
            if !current_or_result {
                result = false;
            }
        }

        Ok(result)
    }

    pub fn header(&self) -> &GreqHeader {
        &self.header
    }

    pub fn content(&self) -> &GreqContent {
        &self.content
    }

    pub fn footer(&self) -> &GreqFooter {
        &self.footer
    }
}
