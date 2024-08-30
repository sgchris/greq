use crate::greq_object::greq_http_request::GreqHttpRequest;
use std::collections::HashMap;

#[derive(Debug)]
struct GreqContents {
    original_string: String,

    http_request: GreqHttpRequest,
}

impl GreqContents {
    pub fn from_string(contents: &str) -> Result<GreqContents, String> {
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
        let uri = request_parts.next().ok_or("Missing URI")?.to_string();
        let _http_version = request_parts.next().ok_or("Missing HTTP version")?;

        // Initialize the HTTP request
        let mut http_request = GreqHttpRequest {
            method,
            uri,
            headers: HashMap::new(),
            content: String::default(),
            hostname: String::default(),
        };

        // Parse headers
        for line in lines.by_ref() {
            if line.trim().is_empty() {
                break; // Empty line signifies the end of headers
            }
            if let Some((key, value)) = line.split_once(':') {
                http_request
                    .headers
                    .insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        // The rest is the content/body
        let content = lines.collect::<Vec<&str>>().join("\r\n");
        http_request.content = content;

        Ok(GreqContents {
            original_string: contents.to_string(),
            http_request,
        })
    }
}
