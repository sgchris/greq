// use crate::greq_object::greq;
use crate::greq_object::greq_content::GreqContent;
use crate::greq_object::greq_footer::GreqFooter;
use crate::greq_object::greq_header::GreqHeader;
use crate::greq_object::greq_response::GreqResponse;
use crate::greq_object::greq_parser::{ 
    parse_sections, 
    parse_header_section, 
    // is_line_only_from_char 
};
use crate::greq_object::greq_errors::GreqError;
use crate::greq_object::greq_evaluator::GreqEvaluator;
use crate::constants::{DEFAULT_DELIMITER_CHAR};
use futures::future::BoxFuture;
use reqwest::{ Client, Method };
use serde::{Deserialize, Serialize};
use crate::greq_object::traits::enrich_with_trait::EnrichWith;
use std::fs;
use crate::cli::cli_tools::CliTools;

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
    pub async fn process(
        file_path: &str, // absolute path to the file
        parse_only: Option<bool>, // if true, only parse the file without executing it
    ) -> Result<Self, GreqError> {
        // parse only is false by default
        let parse_only = parse_only.unwrap_or(false);

        // read the file contents
        let raw_file_contents = fs::read_to_string(file_path).map_err(|e| {
            GreqError::ReadGreqFileError { 
                file_path: file_path.to_string(), 
                reason: e.to_string() 
            }
        })?;

        let header_lines = parse_header_section(&raw_file_contents)
            .map_err(|e| GreqError::ParsingHeaderSectionFailed { reason: e.to_string() })?;

        // initial parsing of the header, to extract the delimiter, the base 
        // request that this greq extends, and the dependency request
        let greq_basic_header = GreqHeader::parse(&header_lines, file_path, None, None)
            .map_err(|e| GreqError::ParsingHeaderSectionFailed { reason: e.to_string() })?;

        // TODO: place the following segment in separate method
        let mut dependency_execution_result_option: Option<GreqResponse> = None;
        let mut base_request_option: Option<Greq> = None;
        if greq_basic_header.extends.is_some() || greq_basic_header.depends_on.is_some() {

            // load the dependency Greq file, and execute it if needed
            if !parse_only {
                if let Some(dependency_greq_file) = greq_basic_header.depends_on {
                    // execute the dependency Greq file
        //Box::pin(self.send_request_and_evaluate_response(show_response))
                    let dependency_greq = Box::pin(Greq::process(
                        &dependency_greq_file,
                        Some(true) // parse the dependency file only
                    )).await.map_err(|e| {
                            GreqError::ExecuteDependencyGreqFileError { 
                                file_path: dependency_greq_file.to_string(), 
                                reason: e.to_string() 
                            }
                        })?;

                    // execute the dependency Greq file if parse_only is false
                    // TODO: pass the correct "show response" value
                    let show_response = false;
                    let dependency_execution_result = dependency_greq
                        .send_request_and_evaluate_response(show_response)
                        .await
                        .map_err(|e| {
                            GreqError::ExecuteDependencyGreqFileError { 
                                file_path: dependency_greq_file.to_string(), 
                                reason: e.to_string() 
                            }
                        })?;

                    // set for later use as a parameter for the current Greq
                    dependency_execution_result_option = Some(dependency_execution_result);
                }
            }

            if let Some(base_request) = greq_basic_header.extends {
                let base_greq = Box::pin(Greq::process(
                    &base_request,
                    Some(true) // parse the base request only
                )).await.map_err(|e| GreqError::ErroLoadingBaseRequest { 
                        file_path: base_request.to_string(), 
                        reason: e.to_string() 
                    })?;

                base_request_option = Some(base_greq);
            }
        }

        // parse the lines and place them in 3 strings vectors
        let sections = parse_sections(&raw_file_contents, greq_basic_header.delimiter)?;

        // re-parse the header lines with the base request and dependency execution result
        let raw_header_lines = sections.get(0)
            .ok_or(GreqError::ParsingHeaderSectionFailed { 
                reason: "Header section is missing".to_string() 
            })?;
        let greq_header = GreqHeader::parse(
            &raw_header_lines, 
            file_path,
            base_request_option.as_ref().map(|g| &g.header),
            dependency_execution_result_option.as_ref()
        ).map_err(|e| GreqError::ParsingHeaderSectionFailed { reason: e.to_string() })?;

        // parse the content section
        let raw_content_lines = sections.get(1)
            .ok_or(GreqError::ParsingContentSectionFailed {
                reason: "Content section is missing".to_string() 
            })?;
        let greq_content = GreqContent::parse(
            &raw_content_lines,
            base_request_option.as_ref().map(|g| &g.content),
            dependency_execution_result_option.as_ref()
        ).map_err(|e| GreqError::ParsingHeaderSectionFailed { reason: e.to_string() })?;

        // parse the footer section
        let raw_footer_lines = sections.get(2)
            .ok_or(GreqError::ParsingFooterSectionFailed {
                reason: "footer section is missing".to_string() 
            })?;
        let greq_footer = GreqFooter::parse(
            &raw_footer_lines,
            base_request_option.as_ref().map(|g| &g.footer),
            dependency_execution_result_option.as_ref()
        ).map_err(|e| GreqError::ParsingHeaderSectionFailed { reason: e.to_string() })?;

        let greq = Greq {
            file_contents: raw_file_contents,
            sections_delimiter: greq_basic_header.delimiter,
            header: greq_header,
            content: greq_content,
            footer: greq_footer,
        };

        Ok(greq)
    }

    /// execute the request and return the evaluation result and the raw response.
    pub async fn send_request_and_evaluate_response(
        &self,
        show_response: bool
    ) -> Result<GreqResponse, GreqError> {
        let greq_response: GreqResponse = self.get_response().await.map_err(|e| {
            GreqError::HttpError { message: e }
        })?;

        if show_response {
            // if the user wants to see the response, print it
            let response_as_json = serde_json::to_string_pretty(&greq_response)
                .unwrap_or_else(|_| String::from("{}"));
            println!("Response:\r\n{}", response_as_json);
        }

        print!("evaluating conditions... ");
        let evaluation_result_object = self.evaluate(&greq_response);
        if let Err(evaluation_error) = evaluation_result_object {
            CliTools::print_red("failed");
            println!();
            CliTools::print_red(&evaluation_error.to_string());

            return Err(evaluation_error);
        } else {
            CliTools::print_green("done");
        }

        let evaluation_result = evaluation_result_object.unwrap();

        Ok(greq_response)
    }

    /// send an HTTP request using Reqwest and return the response.
    pub async fn get_response(&self) -> Result<GreqResponse, String> {
        // Create a reqwest client
        let client = Client::new();

        // Set up the request builder based on the method in `Greq`
        let full_url = self.get_full_url();

        // start building the request
        let reqwest_method = self.content.method.parse::<Method>().map_err(|e| e.to_string())?;
        let mut request_builder = client.request(reqwest_method, &full_url);

        // Add headers if any
        for (header_key, header_value) in &self.content.headers {
            if Self::should_remove_header_for_reqwest(header_key) {
                // Skip headers that should not be sent
                continue;
            }

            request_builder = request_builder.header(header_key, header_value);
        }

        // Add the body if it's a POST, PUT, or PATCH request
        if !self.content.body.is_empty() && self.request_can_send_body() {
            request_builder = request_builder.body(self.content.body.clone());
        }

        // Execute the request
        print!("sending request to '{}'...", full_url);
        let start_time = std::time::Instant::now();
        let raw_response = request_builder.send().await;
        let elapsed_time = start_time.elapsed().as_millis() as u64;
        if let Err(response_err) = raw_response {
            CliTools::print_red("failed");
            CliTools::print_red(&response_err.to_string());
            return Err(response_err.to_string());
        }

        CliTools::print_green(&format!("done in {} ms", elapsed_time));
        let response = raw_response.unwrap();

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

    /// Evaluate the conditions in the footer of the Greq object.
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
                        original_condition: current_or_group_failures.join(" OR "),
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
                    current_or_group_failures.push(format!("{:?}", condition));
                }

                current_or_group.push(condition_result);
            } else {
                // if the condition is not part of an OR group, evaluate it directly
                if condition_result == false {
                    return Err(GreqError::ConditionEvaluationFailed {
                        original_condition: condition.original_line.clone(),
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
    fn boxed_execute(&self, show_response: bool) -> BoxFuture<'_, Result<GreqResponse, GreqError>> {
        Box::pin(self.send_request_and_evaluate_response(show_response))
    }

    /// Get the full URL of the request, including protocol, host, custom port if needed, and URI.
    pub fn get_full_url(&self) -> String {
        let protocol = if self.header.is_http { "http" } else { "https" };
        let host = &self.content.hostname;

        // add the port if it's not the default one
        let mut port = String::new();
        if let Some(custom_port) = self.content.custom_port { 
            port = format!(":{}", custom_port)
        }

        // ensure the URI starts with a / slash
        let uri = if self.content.uri.is_empty() { 
            String::new()
        } else {
            format!("/{}", &self.content.uri.trim_start_matches('/'))
        };

        format!("{}://{}{}{}", protocol, host, port, uri)
    }

    /// Check if the request can send a body based on the HTTP method.
    #[inline]
    fn request_can_send_body(&self) -> bool {
        matches!(self.content.method.as_str(), "POST" | "PUT" | "PATCH")
    }

    /// Check if a header should be removed for reqwest because it's handled
    /// by the library or is not relevant for the request.
    #[inline]
    pub fn should_remove_header_for_reqwest(header_name: &str) -> bool {
        matches!(header_name.to_lowercase().as_str(),
            "host" | "connection" | "keep-alive" | "content-length" |
            "transfer-encoding" | "proxy-connection" | "proxy-authorization" |
            ":method" | ":path" | ":scheme" | ":authority"
        )
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
