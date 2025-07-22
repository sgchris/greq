use std::{collections::HashMap, intrinsics::const_eval_select};
use regex::Regex;
use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use crate::greq_object::{
    greq::Greq, greq_parser::{
        replace_placeholders_in_lines, resolve_and_check_file_exists, strs_to_cows
    }, 
    greq_response::GreqResponse, 
    traits::enrich_with_trait::EnrichWith
};
use thiserror::Error;
use crate::constants::{DEFAULT_HTTP_VERSION, NEW_LINE};

#[derive(Debug, PartialEq, Error)]
pub enum GreqContentError {
    #[error("Invalid request line format: {line}")]
    InvalidRequestLine { line: String },
    #[error("Content cannot be empty")]
    EmptyContent,
    #[error("Missing HTTP method")]
    MissingHttpMethod,
    #[error("Invalid HTTP method {method}")]
    InvalidHttpMethod { method: String },
    #[error("Invalid URI in the request {uri}")]
    InvalidUri { uri: String },
    #[error("Missing URI")]
    MissingUri,
    #[error("Invalid HTTP version format")]
    InvalidHttpVersion,
    #[error("Missing host header")]
    MissingHost,
    #[error("Invalid port in line: '{line}'")]
    InvalidPort { line: String },
    #[error("Invalid header line format: '{line}'")]
    InvalidHeaderLine { line: String },
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct GreqContent {
    pub method: String,
    pub hostname: String,
    pub custom_port: Option<u16>,
    pub http_version: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl GreqContent {
    pub fn parse(
        content_lines: &Vec<&str>,
        base_request_content: Option<&GreqContent>,
        dependency_response: Option<&GreqResponse>,
    ) -> Result<Self, GreqContentError> {

        // convert COW (changeable on write) strings
        let mut cow_lines = strs_to_cows(content_lines);

        // parse the content lines and initialize the GreqContent object
        let mut greq_content = GreqContent::parse_lines_into_greq_content_object(&cow_lines)?;

        if let Some(base_request_content_obj) = base_request_content {
            // enrich with base_request
            greq_content.enrich_with(base_request_content_obj)
                .map_err(|e| GreqContentError::InvalidHeaderLine { line: e })?;

            // replace placeholders after the enrichment
            if let Some(dependency_response_obj) = dependency_response {
                replace_placeholders_in_lines(&mut cow_lines, dependency_response_obj);
            }
        }

        Ok(greq_content)
    }


    /// Content lines might be imcomplete in case when the "extends" header is used
    /// Therefore, the validation isn't strict.
    fn parse_lines_into_greq_content_object(
        content_lines: &Vec<Cow<str>>
    ) -> Result<Self, GreqContentError> {
        // if no content lines, return the default
        if content_lines.is_empty() {
            return Ok(GreqContent::default());
        }

        // parse the first line to get the method, URI, and HTTP version
        let (
            method, 
            uri, 
            http_version
        ) = GreqContent::parse_request_line(&content_lines[0].trim())?;

        // Initialize the HTTP request
        let mut greq_content = GreqContent {
            method: method.to_string(),
            uri: uri.to_string(),
            http_version: http_version.to_string(),
            headers: HashMap::new(),
            custom_port: None,

            ..Default::default()
        };

        // Start the parsing of headers and content
        let mut body_lines: Vec<&str> = vec![];

        // indicate if we are in the content section (after two newlines)
        let mut reached_request_body = false;

        // iterate starting from the second line
        for line in content_lines[1..].iter() {

            // after the first empty line, we are in the content section
            if line.trim().is_empty() && !reached_request_body {
                reached_request_body = true;
                continue; // Empty line signifies the end of headers
            }

            if reached_request_body {
                body_lines.push(line)
            } else if let Some((key, value)) = line.split_once(':').map(|(k, v)| (k.trim(), v.trim())) {
                GreqContent::parse_header_line(
                    &mut greq_content, 
                    key, 
                    value, 
                    line
                )?;
            } else {
                return Err(GreqContentError::InvalidHeaderLine {
                    line: line.to_string(),
                });
            }
        }

        // The rest is the content/body
        //let content = content_lines.collect::<Vec<&str>>().join("\r\n");
        greq_content.body = body_lines.join(NEW_LINE);

        Ok(greq_content)
    }

    // parse a single header line and update the GreqContent object
    // if the header is "host", it also updates the hostname and port
    fn parse_header_line(
        greq_content: &mut GreqContent,
        key: &str,
        value: &str,
        line: &str
    ) -> Result<(), GreqContentError> {
        greq_content
            .headers
            .insert(key.to_string(), value.to_string());

        // check the special case of "host" header
        if key.to_lowercase() == "host" {
            // check if port supplied
            let hostname: &str;
            let mut port_string: &str = "";
            if value.contains(":") {
                (hostname, port_string) = value.split_once(":")
                    .map_or(("", ""), |value_part| (value_part.0.trim(), value_part.1.trim()));
            } else {
                hostname = value.trim();
            }

            if !hostname.is_empty() {
                greq_content.hostname = hostname.to_string();
            } else {
                return Err(GreqContentError::MissingHost);
            }

            if !port_string.is_empty() {
                let parsed_port = port_string
                    .trim()
                    .parse::<u16>();
                if parsed_port.is_err() {
                    return Err(GreqContentError::InvalidPort {
                        line: line.to_string(),
                    });
                }

                greq_content.custom_port = Some(parsed_port.unwrap());
            }
        }

        Ok(())
    }

    /// Parses the first line of the request to extract method, URI, and HTTP version.
    fn parse_request_line(request_line: &str) -> Result<(&str, &str, &str), GreqContentError> {
        // Parse the request line.
        // (the first line of the content, e.g. "GET /index.html HTTP/1.1")
        let request_parts = request_line.split_whitespace().collect::<Vec<&str>>();

        // check that it has at least 2 parts (method and URI)
        if request_parts.len() < 2 || request_parts.len() > 3 {
            return Err(GreqContentError::InvalidRequestLine {
                line: request_line.to_string(),
            });
        }

        // parse and validate the method (GET/POST/...)
        let method = request_parts[0];
        if !Self::method_is_valid(method) {
            return Err(GreqContentError::InvalidHttpMethod { method: method.to_string() });
        }

        // Parse the URI
        let uri = request_parts[1];

        // Check for invalid characters (basic validation)
        // Allow alphanumeric, path separators, query params, and common URL-safe chars
        let re = Regex::new(r"^[a-zA-Z0-9\-\._~:/\?#\[\]@!$&'\(\)\*\+,;=%]+$").unwrap();
        if !re.is_match(uri) {
            return Err(GreqContentError::InvalidUri { uri: uri.to_string() });
        }

        // Parse the HTTP version
        let http_version = if request_parts.len() > 2 {
            request_parts[2]
        } else {
            DEFAULT_HTTP_VERSION
        };

        Ok((method, uri, http_version))
    }
}

impl EnrichWith for GreqContent {
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
where
        Self: Sized,
    {
        // Update hostname only if current is empty
        if self.hostname.is_empty() {
            self.hostname = object_to_merge.hostname.clone();
        }

        // Update custom_port only if current is None
        if self.custom_port.is_none() {
            self.custom_port = object_to_merge.custom_port;
        }

        // Merge headers only for keys that don't exist in current headers
        for (key, value) in &object_to_merge.headers {
            if !self.headers.contains_key(key) {
                self.headers.insert(key.clone(), value.clone());
            }
        }

        // Update body only if current is empty
        if self.body.is_empty() {
            self.body = object_to_merge.body.clone();
        }

        Ok(())
    }
}

impl GreqContent {
    /// Validates if the provided HTTP method is one of the standard methods.
    pub fn method_is_valid(method: &str) -> bool {
        let valid_methods = [
            "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "TRACE", "CONNECT"
        ];

        valid_methods.contains(&method)
    }

    /// Validates if the provided HTTP version is in the correct format "HTTP/x.y"
    pub fn is_valid_http_version(version: &str) -> bool {
        // Define the regex pattern for "HTTP/x.y" format
        let re = Regex::new(r"^HTTP/\d\.\d$").unwrap();
        re.is_match(version)
    }
}


