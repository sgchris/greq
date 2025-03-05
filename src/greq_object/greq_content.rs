use crate::greq_object::greq_http_request::GreqHttpRequest;
use std::collections::HashMap;
use std::str::FromStr;
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::greq_object::traits::enrich_with_trait::EnrichWith;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GreqContent {
    pub original_string: String,
    pub http_request: GreqHttpRequest,
}

#[derive(Debug, PartialEq)]
pub enum GreqContentErrorCodes {
    EmptyContent,
    MissingRequestLine,
    MissingHttpMethod,
    MissingUri,
    InvalidHttpVersion,
    MissingHost,
    InvalidHeaderLine,
}

#[derive(Debug)]
pub struct GreqContentError {
    pub code: GreqContentErrorCodes,
    pub message: String
}

impl GreqContentErrorCodes {
    pub fn error_message(&self) -> &'static str {
        match self {
            GreqContentErrorCodes::EmptyContent => "Empty content.",
            _ => "Unrecognized content error",
        }
    }
}

impl GreqContentError {
    pub fn new(code: GreqContentErrorCodes, message: &str) -> Self {
        Self { code, message: message.to_string() }
    }

    pub fn from_error_code(code: GreqContentErrorCodes) -> Self {
        let error_message = code.error_message();
        Self::new(code, error_message)
    }
}

impl FromStr for GreqContent {
    type Err = GreqContentError;

    fn from_str(contents: &str) -> Result<GreqContent, Self::Err> {
        // empty contents are not allowed
        if contents.trim().is_empty() {
            return Err(GreqContentError::from_error_code(GreqContentErrorCodes::EmptyContent));
        }

        let mut lines = contents.lines();

        // Parse the request line
        let request_line = lines.next().ok_or(GreqContentError::from_error_code(GreqContentErrorCodes::MissingRequestLine))?;
        let mut request_parts = request_line.split_whitespace();

        // parse the method (GET/POST/...)
        let method = request_parts
            .next()
            .ok_or(GreqContentError::from_error_code(GreqContentErrorCodes::MissingHttpMethod))?
            .to_string();
        if !Self::method_is_valid(&method) {
            return Err(GreqContentError::from_error_code(GreqContentErrorCodes::MissingHttpMethod));
        }

        // Parse the URI
        let uri = request_parts.next().ok_or(GreqContentError::from_error_code(GreqContentErrorCodes::MissingUri))?.to_string();

        // Parse the HTTP version
        let http_version = request_parts.next().unwrap_or("HTTP/1.1").to_string();
        if !Self::is_valid_http_version(&http_version) {
            return Err(GreqContentError::from_error_code(GreqContentErrorCodes::InvalidHttpVersion));
        }

        // Initialize the HTTP request
        let mut http_request = GreqHttpRequest {
            method,
            uri,
            http_version,
            headers: HashMap::new(),
            ..Default::default()
        };

        // Parse the headers and the content
        let mut content_lines: Vec<&str> = vec![];
        let mut is_content_line = false;
        for line in lines.by_ref() {
            if line.trim().is_empty() && !is_content_line {
                is_content_line = true;
                continue; // Empty line signifies the end of headers
            }

            if is_content_line {
                content_lines.push(line)
            } else if let Some((key, value)) = line.split_once(':') {
                http_request
                    .headers
                    .insert(key.trim().to_string(), value.trim().to_string());

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
                        http_request.hostname = hostname.to_string();
                    } else {
                        return Err(GreqContentError::from_error_code(GreqContentErrorCodes::MissingHost));
                    }

                    if !port_string.is_empty() {
                        http_request.port = port_string.trim().parse::<u16>().unwrap_or_else(|_| 443);
                    }
                }
            } else {
                return Err(GreqContentError::from_error_code(GreqContentErrorCodes::InvalidHeaderLine));
            }
        }

        // The rest is the content/body
        //let content = lines.collect::<Vec<&str>>().join("\r\n");
        http_request.content = content_lines.join("\r\n");

        Ok(GreqContent {
            original_string: contents.to_string(),
            http_request,
        })
    }
}

impl EnrichWith for GreqContent {
    fn enrich_with(&mut self, _object_to_merge: &Self) -> Result<(), String>
    where
        Self: Sized,
    {
        // If self is empty, copy everything from object_to_merge
        if self.original_string.is_empty() {
            self.original_string = object_to_merge.original_string.clone();
            self.http_request = object_to_merge.http_request.clone();
            return Ok(());
        }

        // Merge HTTP request
        self.http_request.enrich_with(&object_to_merge.http_request)?;

        // Update original string to reflect the merged state
        self.original_string = format!(
            "{} {}\r\n{}\r\n{}",
            self.http_request.method,
            self.http_request.uri,
            self.http_request.headers.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\r\n"),
            if !self.http_request.content.is_empty() {
                format!("\r\n{}", self.http_request.content)
            } else {
                String::new()
            }
        );

        Ok(())
    }
}

impl GreqContent {
    fn method_is_valid(method: &str) -> bool {
        let valid_methods = [
            "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "TRACE", "CONNECT"
        ];

        //valid_methods.contains(&method.as_str())
        valid_methods.contains(&method)
    }

    fn is_valid_http_version(version: &str) -> bool {
        // Define the regex pattern for "HTTP/x.y" format
        let re = Regex::new(r"^HTTP/\d\.\d$").unwrap();
        re.is_match(version)
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_contents() {
        let result = GreqContent::from_str("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, GreqContentErrorCodes::EmptyContent);
    }

    #[test]
    fn test_only_request_line() {
        let contents = "GET /index.html HTTP/1.1";
        let result = GreqContent::from_str(contents);
        assert!(result.is_ok());

        let greq = result.unwrap();
        assert_eq!(greq.http_request.method, "GET");
        assert_eq!(greq.http_request.uri, "/index.html");
        assert!(greq.http_request.headers.is_empty());
        assert_eq!(greq.http_request.content, "");
    }

    #[test]
    fn test_request_with_headers() {
        let contents = "GET /index.html HTTP/1.1\r\nHost: localhost\r\nUser-Agent: curl";
        let result = GreqContent::from_str(contents);
        assert!(result.is_ok());

        let greq = result.unwrap();
        assert_eq!(greq.http_request.method, "GET");
        assert_eq!(greq.http_request.uri, "/index.html");
        assert_eq!(greq.http_request.headers.get("Host").unwrap_or(&"".to_string()), "localhost");
        assert_eq!(greq.http_request.headers.get("User-Agent").unwrap_or(&"".to_string()), "curl");
        assert_eq!(greq.http_request.content, "");
    }

    #[test]
    fn test_request_with_content() {
        let contents = "GET /index.html HTTP/1.1\r\n\r\nThis is the body content";
        let result = GreqContent::from_str(contents);
        assert!(result.is_ok());

        let greq = result.unwrap();
        assert_eq!(greq.http_request.method, "GET");
        assert_eq!(greq.http_request.uri, "/index.html");
        assert!(greq.http_request.headers.is_empty());
        assert_eq!(greq.http_request.content, "This is the body content");
    }

    #[test]
    fn test_request_with_headers_and_content() {
        let contents = "GET /index.html HTTP/1.1\r\nHost: localhost\r\n\r\nThis is the body content";
        let result = GreqContent::from_str(contents);
        assert!(result.is_ok());

        let greq = result.unwrap();
        assert_eq!(greq.http_request.method, "GET");
        assert_eq!(greq.http_request.uri, "/index.html");
        assert_eq!(greq.http_request.headers.get("Host").unwrap_or(&"".to_string()), "localhost");
        assert_eq!(greq.http_request.content, "This is the body content");
    }

    #[test]
    fn test_request_with_headers_and_double_empty_line_before_content() {
        let contents = "GET /index.html HTTP/1.1\r\nHost: localhost\r\n\r\n\r\nThis is the body content";
        let result = GreqContent::from_str(contents);
        assert!(result.is_ok());

        let greq = result.unwrap();
        assert_eq!(greq.http_request.method, "GET");
        assert_eq!(greq.http_request.uri, "/index.html");
        assert_eq!(greq.http_request.headers.get("Host").unwrap(), "localhost");
        assert_eq!(greq.http_request.content, "\r\nThis is the body content");
    }

    // Additional Tests

    #[test]
    fn test_request_with_malformed_header() {
        let contents = "GET /index.html HTTP/1.1\r\nMalformedHeaderWithoutColon";
        let result = GreqContent::from_str(contents);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, GreqContentErrorCodes::InvalidHeaderLine);
    }

    #[test]
    fn test_missing_method() {
        let contents = "/index.html HTTP/1.1";
        let result = GreqContent::from_str(contents);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, GreqContentErrorCodes::MissingHttpMethod);
    }

    #[test]
    fn test_missing_uri() {
        let contents = "GET";
        let result = GreqContent::from_str(contents);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, GreqContentErrorCodes::MissingUri);
    }

    #[test]
    fn test_http_version() {
        // Case 1: With HTTP version explicitly provided
        let contents_with_version = "GET /index.html HTTP/2.0\r\nHost: localhost";
        let result_with_version = GreqContent::from_str(contents_with_version);
        assert!(result_with_version.is_ok());

        let greq_with_version = result_with_version.unwrap();
        assert_eq!(greq_with_version.http_request.method, "GET");
        assert_eq!(greq_with_version.http_request.uri, "/index.html");
        assert_eq!(greq_with_version.http_request.http_version, "HTTP/2.0");

        // Case 2: Without HTTP version (should default to HTTP/1.1)
        let contents_without_version = "GET /index.html\r\nHost: localhost";
        let result_without_version = GreqContent::from_str(contents_without_version);
        assert!(result_without_version.is_ok());

        let greq_without_version = result_without_version.unwrap();
        assert_eq!(greq_without_version.http_request.method, "GET");
        assert_eq!(greq_without_version.http_request.uri, "/index.html");
        assert_eq!(greq_without_version.http_request.http_version, "HTTP/1.1");
    }

}
