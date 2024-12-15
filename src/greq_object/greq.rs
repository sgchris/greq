use crate::greq_object::greq_content::GreqContent;
use crate::greq_object::greq_footer::GreqFooter;
use crate::greq_object::greq_header::GreqHeader;
use crate::greq_object::greq_http_request::GreqHttpRequest;
use crate::greq_object::greq_response::GreqResponse;
use futures::future::BoxFuture;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::str::FromStr;
use crate::greq_object::traits::enrich_with_trait::EnrichWith;

#[derive(Serialize, Deserialize, Debug)]
pub struct SectionsDelimiter { value: char }
impl Default for SectionsDelimiter { fn default() -> Self { SectionsDelimiter { value: '=' } } }



#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Greq {
    file_contents: String,
    sections_delimiter: SectionsDelimiter,
    header: GreqHeader,
    content: GreqContent,
    footer: GreqFooter,
}

// possible Greq parsing errors
#[derive(Debug, PartialEq)]
pub enum GreqErrorCodes {
    TooFewSections,
    TooManySections,
    MissingSections,
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut greq = Greq {
            file_contents: s.to_string(),
            ..Default::default()
        };

        let mut lines = s.lines();
        let mut sections: [Vec<&str>; 3] = [vec![], vec![], vec![]];

        // try to extract custom delimiter, if provided
        if let Some(delimiter_line) = lines.find(|line| line.to_lowercase().starts_with("delimiter")) {
            if let Some((_, value)) = delimiter_line.split_once(':') {
                greq.sections_delimiter.value = value.trim().chars().next().unwrap_or('=');
            }
        }

        let mut part_number: usize = 0;
        lines.try_for_each(|line| {
            // check for delimiter
            if line.starts_with(&greq.sections_delimiter.value.to_string().repeat(4)) {
                part_number += 1;
            } else if part_number > 2 {
                return Err(GreqError::from_error_code(GreqErrorCodes::TooManySections));
            } else {
                sections[part_number].push(line);
            }

            Ok(())
        })?;

        if part_number != 2 {
            return Err(GreqError::from_error_code(GreqErrorCodes::TooFewSections));
        }

        print!("parsing header...");
        greq.header = GreqHeader::from_str(&sections[0].join("\r\n"))
            .map_err(|e| GreqError::from_error_code(GreqErrorCodes::ParsingHeaderSectionFailed))?;
        println!("done");

        print!("parsing content...");
        greq.content = GreqContent::from_str(&sections[1].join("\r\n"))
            .map_err(|e| GreqError::from_error_code(GreqErrorCodes::ParsingContentSectionFailed))?;
        // set the default protocol to https
        greq.content.http_request.is_http = greq.header.is_http.unwrap_or(false);
        println!("done");

        print!("parsing footer...");
        greq.footer = GreqFooter::from_str(&sections[2].join("\r\n"))
            .map_err(|e| GreqError::from_error_code(GreqErrorCodes::ParsingFooterSectionFailed))?;
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

        let evaluation_result = self.evaluate().unwrap_or(false);
        Ok((Some(evaluation_result), result.body.unwrap()))
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

    pub fn from_file(file_path: &str) -> Result<Greq, GreqError> {
        let read_file_result = Self::read_file_to_string(file_path);
        if read_file_result.is_err() {
            let err_message = read_file_result.err().unwrap().to_string();
            return Err(GreqError::new(GreqErrorCodes::ReadGreqFileError, &err_message));
        }

        let mut greq = Greq::from_str(read_file_result.unwrap().as_str())?;

        // TODO: avoid dependency loop
        if let Some(ref base_request_path) = greq.header.base_request {
            let base_greq = Greq::from_file(base_request_path)?;
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

    fn read_file_to_string<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
        let mut file = File::open(file_path)?; // Open the file
        let mut contents = String::new();
        file.read_to_string(&mut contents)?; // Read the file contents into the string
        Ok(contents) // Return the contents as a result
    }

    fn evaluate(&self) -> Result<bool, String> {
        Ok(true)
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

        let parse_result = Greq::from_str(input);
        assert!(parse_result.is_ok(), "Valid input should succeed");
        let greq = parse_result.unwrap();

        assert_eq!(
            greq.header.original_string,
            "output-folder: /path/to/output/folder\r\nproject: greq test"
        );
        assert_eq!(
            greq.content.original_string,
            "GET /some-url\r\nhost: greq-test.example.com"
        );
        assert_eq!(
            greq.footer.original_string,
            "status-code: 200\r\nresponse-content contains: greq test result"
        );
    }

    #[test]
    fn test_missing_sections() {
        let input = r#"output-folder: /path/to/output/folder
project: greq test
====
GET /some-url
host: greq-test.example.com"#; // Missing footer section

        let parse_result = Greq::from_str(input);
        assert!(
            parse_result.is_err(),
            "Missing sections should cause an error"
        );
        assert_eq!(parse_result.unwrap_err().code, GreqErrorCodes::MissingSections);
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

        let parse_result = Greq::from_str(input);
        assert!(
            parse_result.is_err(),
            "More than 3 sections should cause an error"
        );
        assert_eq!(parse_result.unwrap_err().code, GreqErrorCodes::TooManySections);
    }

    #[test]
    fn test_empty_input() {
        let input = "";

        let parse_result = Greq::from_str(input);
        assert!(parse_result.is_err(), "Empty input should cause an error");
        assert_eq!(parse_result.unwrap_err().code, GreqErrorCodes::TooFewSections);
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
        let parse_result = Greq::from_str(input);
        assert!(
            parse_result.is_err(),
            "Header parse error should be propagated"
        );
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
        let parse_result = Greq::from_str(input);
        assert!(
            parse_result.is_err(),
            "Content parse error should be propagated"
        );
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
        let parse_result = Greq::from_str(input);
        assert!(
            parse_result.is_err(),
            "Footer parse error should be propagated"
        );
    }

    #[test]
    fn test_section_delimiter_handling() {
        let input = r#"output-folder: /path/to/output/folder
====
GET /some-url
====
status-code: 200
response-content contains: greq test result"#; // No header content, but valid delimiters

        let parse_result = Greq::from_str(input);
        assert!(
            parse_result.is_ok(),
            "Valid delimiters but no content should still be okay"
        );
        let greq = parse_result.unwrap();

        assert_eq!(
            greq.header.original_string,
            "output-folder: /path/to/output/folder"
        ); // Header is empty
        assert_eq!(greq.content.original_string, "GET /some-url"); // Content is valid
        assert_eq!(
            greq.footer.original_string,
            "status-code: 200\r\nresponse-content contains: greq test result"
        ); // Footer is valid
    }
}
