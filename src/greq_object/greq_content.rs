use crate::greq_object::greq_http_request::GreqHttpRequest;
use crate::greq_object::traits::from_string_trait::FromString;
use std::collections::HashMap;
use regex::Regex;
use crate::greq_object::traits::enrich_with_trait::EnrichWith;
use crate::json_string;

#[derive(Debug, Default)]
pub struct GreqContent {
    pub original_string: String,
    pub http_request: GreqHttpRequest,
}

impl FromString for GreqContent {
    fn from_string(contents: &str) -> Result<GreqContent, String> {
        // empty contents are not allowed
        if contents.trim().is_empty() {
            return Err("empty contents".to_string());
        }

        let mut lines = contents.lines();

        // Parse the request line
        let request_line = lines.next().ok_or("Missing request line")?;
        let mut request_parts = request_line.split_whitespace();
        let method = request_parts
            .next()
            .ok_or("Missing HTTP method")?
            .to_string();
        if !Self::method_is_valid(&method) {
            return Err("Missing HTTP method".to_string());
        }

        let uri = request_parts.next().ok_or("Missing URI")?.to_string();
        if Self::is_valid_http_version(&uri) {
            return Err("Missing URI".to_string());
        }

        let http_version = request_parts.next().unwrap_or("HTTP/1.1").to_string();

        // Initialize the HTTP request
        let mut http_request = GreqHttpRequest {
            method,
            uri,
            http_version,
            headers: HashMap::new(),
            ..Default::default()
        };

        // Parse headers
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
                        return Err(format!("Missing host"));
                    }

                    if !port_string.is_empty() {
                        http_request.port = port_string.trim().parse::<u16>().unwrap_or_else(|_| 443);
                    }
                }
            } else {
                return Err(format!("Invalid header line: {}", line));
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
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
    where
        Self: Sized
    {
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

    pub fn as_string(&self) -> String {
        format!(
            "{{
  \"original_string\": {},
  \"http_request\": {}
}}",
            json_string!(&self.original_string),
            self.http_request.as_string()
        )
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_contents() {
        let result = GreqContent::from_string("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "empty contents".to_string());
    }

    #[test]
    fn test_only_request_line() {
        let contents = "GET /index.html HTTP/1.1";
        let result = GreqContent::from_string(contents);
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
        let result = GreqContent::from_string(contents);
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
        let result = GreqContent::from_string(contents);
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
        let result = GreqContent::from_string(contents);
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
        let result = GreqContent::from_string(contents);
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
        let result = GreqContent::from_string(contents);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid header line: MalformedHeaderWithoutColon".to_string());
    }

    #[test]
    fn test_missing_method() {
        let contents = "/index.html HTTP/1.1";
        let result = GreqContent::from_string(contents);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing HTTP method".to_string());
    }

    #[test]
    fn test_missing_uri() {
        let contents = "GET HTTP/1.1";
        let result = GreqContent::from_string(contents);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing URI".to_string());
    }

    #[test]
    fn test_http_version() {
        // Case 1: With HTTP version explicitly provided
        let contents_with_version = "GET /index.html HTTP/2.0\r\nHost: localhost";
        let result_with_version = GreqContent::from_string(contents_with_version);
        assert!(result_with_version.is_ok());

        let greq_with_version = result_with_version.unwrap();
        assert_eq!(greq_with_version.http_request.method, "GET");
        assert_eq!(greq_with_version.http_request.uri, "/index.html");
        assert_eq!(greq_with_version.http_request.http_version, "HTTP/2.0");

        // Case 2: Without HTTP version (should default to HTTP/1.1)
        let contents_without_version = "GET /index.html\r\nHost: localhost";
        let result_without_version = GreqContent::from_string(contents_without_version);
        assert!(result_without_version.is_ok());

        let greq_without_version = result_without_version.unwrap();
        assert_eq!(greq_without_version.http_request.method, "GET");
        assert_eq!(greq_without_version.http_request.uri, "/index.html");
        assert_eq!(greq_without_version.http_request.http_version, "HTTP/1.1");
    }

}
