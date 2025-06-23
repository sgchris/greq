use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::greq_object::traits::enrich_with_trait::EnrichWith;
use thiserror::Error;
use crate::constants::{DEFAULT_HTTP_VERSION, NEW_LINE, DEFAULT_HTTPS_PORT};

#[derive(Debug, PartialEq, Error)]
pub enum GreqContentError {
    #[error("Content cannot be empty")]
    EmptyContent,
    #[error("Missing request line")]
    MissingRequestLine,
    #[error("Missing HTTP method")]
    MissingHttpMethod,
    #[error("Invalid HTTP method {method}")]
    InvalidHttpMethod { method: String },
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
    pub port: u16,
    pub http_version: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl GreqContent {
    pub fn parse(content_lines: &Vec<&str>) -> Result<Self, GreqContentError> {
        // Check the minimum number of lines.
        // must be at least 2 lines: the request line and the host header.
        if content_lines.len() < 2 {
            return Err(GreqContentError::EmptyContent);
        }

        // Parse the request line.
        // (the first line of the content, e.g. "GET /index.html HTTP/1.1")
        let request_line = content_lines[0].trim();
        let mut request_parts = request_line.split_whitespace();

        // parse and validate the method (GET/POST/...)
        let method = request_parts
            .next()
            .ok_or(GreqContentError::MissingHttpMethod)?
            .to_string();
        if !Self::method_is_valid(&method) {
            return Err(GreqContentError::InvalidHttpMethod { method });
        }

        // Parse the URI
        let uri = request_parts
            .next()
            .ok_or(GreqContentError::MissingUri)?.to_string();

        // Parse the HTTP version
        let http_version = request_parts.next().unwrap_or(DEFAULT_HTTP_VERSION).to_string();
        if !Self::is_valid_http_version(&http_version) {
            return Err(GreqContentError::InvalidHttpVersion);
        }

        // Initialize the HTTP request
        let mut greq_content = GreqContent {
            method,
            uri,
            http_version,
            headers: HashMap::new(),
            port: DEFAULT_HTTPS_PORT,

            ..Default::default()
        };

        // Initialize the HTTP request with the hostname and port
        let mut host_header_exists = false;

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
                greq_content
                    .headers
                    .insert(key.to_string(), value.to_string());

                // check the special case of "host" header
                if key.to_lowercase() == "host" {
                    host_header_exists = true;

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

                        greq_content.port = parsed_port.unwrap();
                    }
                }
            } else {
                return Err(GreqContentError::InvalidHeaderLine {
                    line: line.to_string(),
                });
            }
        }

        // Check if the host header was found
        if !host_header_exists {
            return Err(GreqContentError::MissingHost);
        }

        // The rest is the content/body
        //let content = content_lines.collect::<Vec<&str>>().join("\r\n");
        greq_content.body = body_lines.join(NEW_LINE);

        Ok(greq_content)
    }
}

impl EnrichWith for GreqContent {
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
    where
        Self: Sized,
    {
        // Update method, URI, and body
        self.method = object_to_merge.method.clone();
        self.uri = object_to_merge.uri.clone();
        self.body = object_to_merge.body.clone();
        self.hostname = object_to_merge.hostname.clone();
        self.port = object_to_merge.port;

        // Merge headers
        for (key, value) in &object_to_merge.headers {
            self.headers.insert(key.clone(), value.clone());
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
