use crate::greq_object::greq_content::GreqContent;
use crate::greq_object::greq_footer::GreqFooter;
use crate::greq_object::greq_header::GreqHeader;
use crate::greq_object::greq_response::GreqResponse;
use crate::greq_object::greq_parser::{ extract_delimiter, parse_sections };
use crate::greq_object::greq_errors::GreqError;
use crate::greq_object::greq_evaluator::GreqEvaluator;
use crate::constants::{DEFAULT_DELIMITER_CHAR, DEFAULT_HTTPS_PORT, DEFAULT_HTTP_PORT, NEW_LINE};
use futures::future::BoxFuture;
use reqwest::{ Client, Method };
use serde::{Deserialize, Serialize};
use crate::greq_object::traits::enrich_with_trait::EnrichWith;
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GreqExecutionResult {
    pub succeeded: Option<bool>,

    pub response_code: u16,
    pub response_headers: HashMap<String, String>,
    pub response_body: String,
    pub response_milliseconds: u128,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Greq {
    pub file_contents: String,
    
    pub sections_delimiter: char,

    // the sections of the Greq file
    pub header: GreqHeader,
    pub content: GreqContent,
    pub footer: GreqFooter,
}

impl Greq {
    // Add a new boxed version of the `execute` function to handle recursion.
    pub fn parse(raw_file_contents: &str) -> Result<Self, GreqError> {
        // initialize a new Greq object with the original file contents
        let mut greq = Greq {
            file_contents: raw_file_contents.to_string(),
            ..Default::default()
        };

        // check if a custom delimiter was defined in the file, using "delimiter" header.
        let delimiter_char = extract_delimiter(raw_file_contents).unwrap_or(DEFAULT_DELIMITER_CHAR);
        greq.sections_delimiter = delimiter_char;

        // greq file must have 3 sections. 
        // the header (metadata), content (http raw request), and footer (the evaluation conditions)
        let sections = parse_sections(raw_file_contents, greq.sections_delimiter)?;

        print!("parsing header...");
        greq.header = GreqHeader::parse(&sections[0])
            .map_err(|e| GreqError::ParsingHeaderSectionFailed { reason: e.to_string() })?;
        println!("done");

        print!("parsing content...");
        greq.content = GreqContent::parse(&sections[1])
            .map_err(|e| GreqError::ParsingContentSectionFailed { reason: e.to_string() })?;
        // set the default protocol to https
        println!("done");

        print!("parsing footer...");
        greq.footer = GreqFooter::parse(&sections[2])
            .map_err(|_e| GreqError::ParsingFooterSectionFailed { reason: "Error parsing the footer section".to_string() })?;
        println!("done");

        // update the content with the port
        greq.content.port = if greq.header.is_http { DEFAULT_HTTP_PORT } else { DEFAULT_HTTPS_PORT };

        Ok(greq)
    }

    // Load a Greq object from a file
    pub fn from_file(file_path: &str) -> Result<Greq, GreqError> {
        if !file_path.ends_with(".greq") {
            return Err(GreqError::ReadGreqFileError { 
                file_path: file_path.to_string(),
                reason: "File must have a .greq extension".to_string()
            });
        }

        // check that the file exists and is readable
        if !fs::metadata(file_path).map_err(|e| GreqError::ReadGreqFileError { 
            file_path: file_path.to_string(),
            reason: e.to_string() 
        })?.permissions().readonly() {
            return Err(GreqError::ReadGreqFileError { 
                file_path: file_path.to_string(), 
                reason: "Insufficient permissions to read the file".to_string() 
            });
        }

        let file_contents = fs::read_to_string(file_path).map_err(|e| {
            GreqError::ReadGreqFileError { 
                file_path: file_path.to_string(), 
                reason: e.to_string() 
            }
        })?;

        // parse the file contents into a Greq object
        let mut greq = Greq::parse(&file_contents)?;

        // load the base request if specified
        // "ref" is used to avoid cloning the string unnecessarily
        if let Some(base_request_path) = greq.header.base_request.clone() {
            let base_greq = Greq::from_file(&base_request_path)?;

            // merge the base request into the current request
            greq.enrich_with(&base_greq)
                .map_err(|e| GreqError::EnrichmentFailed { dependency: base_request_path.to_string(), reason: e.to_string() })?;
        }

        Ok(greq)
    }

    /// execute the request and return the evaluation result and the raw response.
    pub async fn execute(&self) -> Result<GreqExecutionResult, GreqError> {
        if let Some(ref depends_on_request_path) = self.header.depends_on {
            // get_response the dependant request
            let dependant_request = Greq::from_file(depends_on_request_path)?;
            dependant_request.boxed_execute().await?;
        }

        let greq_response: GreqResponse = self.get_response().await.map_err(|e| {
            GreqError::HttpError { message: e }
        })?;

        let evaluation_result = self.evaluate(&greq_response)?;

        Ok(GreqExecutionResult {
            succeeded: Some(evaluation_result), 
            // successful
            response_code: greq_response.status_code,
            response_headers: greq_response.headers.clone(),
            response_body: greq_response.body.unwrap_or_default(),
            response_milliseconds: 0, // Placeholder for now, can be set later if needed
        })
    }

    // send an HTTP request using Reqwest and return the response.
    pub async fn get_response(&self) -> Result<GreqResponse, String> {
        // Create a reqwest client
        let client = Client::new();

        // Set up the request builder based on the method in `Greq`
        let full_url = self.get_full_url();

        // start building the request
        println!("sending request to {}.\r\ndetails: {:?}\r\n", full_url, self.content);
        let reqwest_method = self.content.method.parse::<Method>().map_err(|e| e.to_string())?;
        let mut request_builder = client.request(reqwest_method, &full_url);

        // Add headers if any
        for (header_key, header_value) in &self.content.headers {
            request_builder = request_builder.header(header_key, header_value);
        }

        // Add the body if it's a POST, PUT, or PATCH request
        if !self.content.body.is_empty() && self.request_can_send_body() {
            request_builder = request_builder.body(self.content.body.clone());
        }

        // Execute the request
        let start_time = std::time::Instant::now();
        let raw_response = request_builder.send().await;
        let elapsed_time = start_time.elapsed().as_millis() as u64;

        // Check for errors in the response
        let response = raw_response.map_err(|e| e.to_string())?;

        let mut greq_response = GreqResponse {
            status_code: response.status().as_u16(),
            reason_phrase: response.status().canonical_reason().unwrap_or("Unknown").to_string(),
            headers: response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect(),
            body: None, // Body will be set later
            response_milliseconds: elapsed_time,
        };

        // Get the body (if any)
        greq_response.body = match response.text().await {
            Ok(body) => Some(body),
            Err(e) => return Err(format!("Failed to read response body: {}", e)),
        };

        // Return the GreqResponse
        Ok(greq_response)
    }

    fn evaluate(&self, greq_response: &GreqResponse) -> Result<bool, GreqError> {
        // the final result
        let mut result = true;

        // "OR" group handling
        let mut current_or_group: Vec<bool> = Vec::new();
        let mut current_or_group_failures: Vec<String> = Vec::new();
        let mut current_or_result;

        // loop through the conditions in the footer and evaluate one-by-one
        for condition in &self.footer.conditions {
            // check if OR group ended
            if !condition.has_or && !current_or_group.is_empty() {
                current_or_result = current_or_group.iter().any(|&x| x);
                if current_or_result == false {
                    // IR group failed, return
                    return Err(GreqError::ConditionEvaluationFailed {
                        reason: current_or_group_failures.join(NEW_LINE),
                    });
                } else {
                    current_or_group.clear();
                }
            }

            // condition evaluation result
            let condition_result = GreqEvaluator::evaluate(&greq_response, condition);

            // check if the condition is part of an OR group
            if condition.has_or {
                // save the failed condition for later reporting
                if condition_result == false {
                    current_or_group_failures.push(format!("Condition failed: {:?}", condition));
                }

                current_or_group.push(condition_result);
            } else {
                // if the condition is not part of an OR group, evaluate it directly
                if condition_result == false {
                    return Err(GreqError::ConditionEvaluationFailed {
                        reason: format!("Condition failed: {:?}", condition),
                    });
                }
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

    /// Execute the request and return a boxed future.
    /// This is useful for recursive calls where the return type needs to be boxed.
    fn boxed_execute(&self) -> BoxFuture<'_, Result<GreqExecutionResult, GreqError>> {
        Box::pin(self.execute())
    }

    /// Get the full URL of the request, including protocol, host, custom port if needed, and URI.
    fn get_full_url(&self) -> String {
        let protocol = if self.header.is_http { "http" } else { "https" };
        let host = &self.content.hostname;

        // add the port if it's not the default one
        let port: String = match self.content.port {
            DEFAULT_HTTP_PORT | DEFAULT_HTTPS_PORT => "".to_string(),
            _ => format!(":{}", self.content.port),
        };

        let uri = &self.content.uri;

        format!("{}://{}{}{}", protocol, host, port, uri)
    }

    /// Check if the request can send a body based on the HTTP method.
    #[inline]
    fn request_can_send_body(&self) -> bool {
        matches!(self.content.method.as_str(), "POST" | "PUT" | "PATCH")
    }
}

impl EnrichWith for Greq {

    /// Enrich the current Greq object with another Greq object.
    /// used to fill in the missing parts of the request.
    fn enrich_with(&mut self, another_object: &Self) -> Result<(), String>
    where
        Self: Sized,
    {
        self.header.enrich_with(&another_object.header)?;
        self.content.enrich_with(&another_object.content)?;
        self.footer.enrich_with(&another_object.footer)?;

        Ok(())
    }
}
