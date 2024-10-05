use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use reqwest::Client;

use crate::greq_object::greq_content::GreqContent;
use crate::greq_object::greq_footer::GreqFooter;
use crate::greq_object::greq_header::GreqHeader;
use crate::greq_object::greq_http_request::GreqHttpRequest;
use crate::greq_object::greq_response::GreqResponse;

use crate::greq_object::traits::from_string_trait::FromString;
use crate::greq_object::traits::enrich_with_trait::EnrichWith;

#[derive(Debug, Default)]
pub struct Greq {
    file_contents: String,
    header: GreqHeader,
    content: GreqContent,
    footer: GreqFooter,
}

impl FromString for Greq {
    fn from_string(s: &str) -> Result<Self, String> {
        let mut greq = Greq {
            file_contents: s.to_string(),
            ..Default::default()
        };

        let lines = s.lines();
        let mut sections: [Vec<&str>; 3] = [vec![], vec![], vec![]];
        let mut errors: Vec<&str> = vec![];

        let mut part_number: usize = 0;
        lines.for_each(|line| {
            // skip on errors
            if !errors.is_empty() {
                return;
            }

            // check for delimiter
            if line.starts_with("====") {
                part_number += 1;
                return;
            }

            if part_number > 2 {
                errors.push("more than 3 sections");
                return;
            }

            sections[part_number].push(line);
        });

        if errors.is_empty() && part_number != 2 {
            errors.push("missing sections");
        }

        if !errors.is_empty() {
            return Err(errors.join(". "));
        }

        print!("parsing header...");
        greq.header = GreqHeader::from_string(&sections[0].join("\r\n"))?;
        println!("done");

        print!("parsing content...");
        greq.content = GreqContent::from_string(&sections[1].join("\r\n"))?;
        println!("done");

        print!("parsing footer...");
        greq.footer = GreqFooter::from_string(&sections[2].join("\r\n"))?;
        println!("done");

        Ok(greq)
    }
}

impl EnrichWith for Greq {
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
    where
        Self: Sized
    {
        self.header.enrich_with(&object_to_merge.header)?;
        self.content.enrich_with(&object_to_merge.content)?;
        self.footer.enrich_with(&object_to_merge.footer)?;

        Ok(())
    }
}

/*
 */

impl Greq {
    pub async fn execute(&self) -> Result<GreqResponse, String> {
        // Create a reqwest client
        let client = Client::new();

        // Set up the request builder based on the method in `Greq`
        let full_url = self.content.http_request.get_full_url();
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
        let reason_phrase = response.status().canonical_reason().unwrap_or("Unknown").to_string();

        // Collect headers into a HashMap
        let headers = response.headers().iter().map(|(k, v)| {
            (k.to_string(), v.to_str().unwrap_or("").to_string())
        }).collect();

        // Get the body (if any)
        let body = response.text().await.map_err(|e| e.to_string())?;


        // Return the GreqResponse
        Ok(GreqResponse {
            status_code,
            reason_phrase,
            headers,
            body: Some(body.clone())
        })
    }

    pub fn from_file(file_path: &str) -> Result<Greq, String> {
        let read_file_result = Self::read_file_to_string(file_path);
        if read_file_result.is_err() {
            return Err(read_file_result.err().unwrap().to_string());
        }

        let mut greq = Greq::from_string(read_file_result.unwrap().as_str())?;

        // TODO: avoid dependency loop
        if let Some(ref base_request_path) = greq.header.base_request {
            let base_greq = Greq::from_file(base_request_path)?;
            greq.enrich_with(&base_greq)?;
        }

        if let Some(ref depends_on_request_path) = greq.header.depends_on {
            // execute the dependant request
            let dependant_request = Greq::from_file(depends_on_request_path)?;
            let result = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(dependant_request.execute());
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

    fn read_file_to_string<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
        let mut file = File::open(file_path)?; // Open the file
        let mut contents = String::new();
        file.read_to_string(&mut contents)?; // Read the file contents into the string
        Ok(contents) // Return the contents as a result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_input() {
        let input = r#"output-folder: /path/to/output/folder
project: greq test
====
GET /some-url
host: greq-test.example.com
====
status-code: 200
response-content contains: greq test result"#;

        let parse_result = Greq::from_string(input);
        assert!(parse_result.is_ok(), "Valid input should succeed");
        let greq = parse_result.unwrap();

        assert_eq!(greq.header.original_string, "output-folder: /path/to/output/folder\r\nproject: greq test");
        assert_eq!(greq.content.original_string, "GET /some-url\r\nhost: greq-test.example.com");
        assert_eq!(greq.footer.original_string, "status-code: 200\r\nresponse-content contains: greq test result");
    }

    #[test]
    fn test_missing_sections() {
        let input = r#"output-folder: /path/to/output/folder
project: greq test
====
GET /some-url
host: greq-test.example.com"#; // Missing footer section

        let parse_result = Greq::from_string(input);
        assert!(parse_result.is_err(), "Missing sections should cause an error");
        assert_eq!(parse_result.unwrap_err(), "missing sections");
    }

    #[test]
    fn test_extra_sections() {
        let input = r#"output-folder: /path/to/output/folder
project: greq test
====
GET /some-url
host: greq-test.example.com
====
status-code: 200
response-content contains: greq test result
====
Extra section line"#; // More than 3 sections

        let parse_result = Greq::from_string(input);
        assert!(parse_result.is_err(), "More than 3 sections should cause an error");
        assert_eq!(parse_result.unwrap_err(), "more than 3 sections");
    }

    #[test]
    fn test_empty_input() {
        let input = "";

        let parse_result = Greq::from_string(input);
        assert!(parse_result.is_err(), "Empty input should cause an error");
        assert_eq!(parse_result.unwrap_err(), "missing sections");
    }

    #[test]
    fn test_header_parse_error() {
        // Assume GreqHeader has custom parsing logic that can fail
        let input = r#"Invalid header format
====
GET /some-url
host: greq-test.example.com
====
status-code: 200
response-content contains: greq test result"#;

        // Mocking GreqHeader to return an error in this case
        let parse_result = Greq::from_string(input);
        assert!(parse_result.is_err(), "Header parse error should be propagated");
    }

    #[test]
    fn test_content_parse_error() {
        // Assume GreqContent has custom parsing logic that can fail
        let input = r#"output-folder: /path/to/output/folder
project: greq test
====
Invalid content format
====
status-code: 200
response-content contains: greq test result"#;

        // Mocking GreqContent to return an error in this case
        let parse_result = Greq::from_string(input);
        assert!(parse_result.is_err(), "Content parse error should be propagated");
    }

    #[test]
    fn test_footer_parse_error() {
        // Assume GreqFooter has custom parsing logic that can fail
        let input = r#"output-folder: /path/to/output/folder
project: greq test
====
GET /some-url
host: greq-test.example.com
====
Invalid footer format"#;

        // Mocking GreqFooter to return an error in this case
        let parse_result = Greq::from_string(input);
        assert!(parse_result.is_err(), "Footer parse error should be propagated");
    }

    #[test]
    fn test_section_delimiter_handling() {
        let input = r#"output-folder: /path/to/output/folder
====
GET /some-url
====
status-code: 200
response-content contains: greq test result"#; // No header content, but valid delimiters

        let parse_result = Greq::from_string(input);
        assert!(parse_result.is_ok(), "Valid delimiters but no content should still be okay");
        let greq = parse_result.unwrap();

        assert_eq!(greq.header.original_string, "output-folder: /path/to/output/folder"); // Header is empty
        assert_eq!(greq.content.original_string, "GET /some-url"); // Content is valid
        assert_eq!(greq.footer.original_string, "status-code: 200\r\nresponse-content contains: greq test result"); // Footer is valid
    }
}

