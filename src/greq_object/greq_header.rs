use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::greq_object::traits::enrich_with_trait::EnrichWith;

#[derive(Debug, PartialEq, Error)]
pub enum GreqHeaderError {
    #[error("Unknown header '{header_name}' encountered in GreqHeader parsing")]
    UnknownHeader { header_name: String },
    #[error("The line '{line}' does not contain a colon sign")]
    LineHasNoColonSign { line: String },
    #[error("Header has no name before the colon sign: '{line}'")]
    HeaderHasNoName { line: String },
    #[error("Header has no value after the colon sign: '{header_name}'")]
    HeaderHasNoValue { header_name: String },
    #[error("Not valid header value '{line}'")]
    InvalidHeaderValue { line: String },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct GreqHeader {
    // the delimiter character to separate sections. Default '='.
    // (This header is used by the parent Greq object to parse the file.)
    pub delimiter: char,

    pub project: Option<String>,          // the name of the project. Will be implemented.
    pub output_folder: Option<String>,    // path to a destination folder. Default current.
    pub output_file_name: Option<String>, // output filename. default current file name with ".response" extension.

    // http and not https request
    pub is_http: bool,  

    // the request that this file extends
    pub base_request: Option<String>,

    // get_response that request before executing this one
    pub depends_on: Option<String>,
}

// override some default values
impl Default for GreqHeader {
    fn default() -> Self {
        GreqHeader {
            delimiter: '=',
            is_http: false,
            project: None,           // Option<T> defaults to None
            output_folder: None,
            output_file_name: None,
            base_request: None,
            depends_on: None,
        }
    }
}

impl GreqHeader {
    // parse the lines that are related to the header of the request
    pub fn parse(header_lines: &Vec<&str>) -> Result<GreqHeader, GreqHeaderError> {
        // initialize the object
        let mut greq_header = GreqHeader::default();

        // parse the lines and assign properties
        for line in header_lines.iter() {

            // trim whitespace from the line
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Ensure the line contains a colon
            let (header_name, header_value) = line
                .split_once(":")
                .map(|(n, v)| (n.trim(), v.trim()))
                .ok_or_else(|| GreqHeaderError::LineHasNoColonSign { line: line.to_string() })?;

            // check if the header name is empty
            if header_name.is_empty() {
                return Err(GreqHeaderError::HeaderHasNoName { line: line.to_string() });
            }

            // check if the header value is empty
            if header_value.is_empty() {
                return Err(GreqHeaderError::HeaderHasNoValue { header_name: header_name.to_string() });
            }

            match header_name.to_lowercase().as_str() {
                "delimiter" => {
                    if header_value.len() != 1 {
                        return Err(GreqHeaderError::InvalidHeaderValue { line: line.to_string() });
                    }

                    greq_header.delimiter = header_value.chars().next().unwrap();
                }
                "project" => {
                    greq_header.project = Some(header_value.to_string());
                }
                "output-folder" => {
                    greq_header.output_folder = Some(header_value.to_string());
                }
                "output-file-name" => {
                    greq_header.output_file_name = Some(header_value.to_string());
                }
                "is-http" => {
                    greq_header.is_http = match header_value.to_lowercase().as_str() {
                        "true" | "yes" | "1" => true,
                        "false" | "no" | "0" => false,
                        _ => return Err(GreqHeaderError::InvalidHeaderValue { line: line.to_string() }),
                    };
                }
                "base-request" => {
                    greq_header.base_request = Some(header_value.to_string());
                }
                "depends-on" => {
                    greq_header.depends_on = Some(header_value.to_string());
                }
                _ => {
                    return Err(GreqHeaderError::UnknownHeader { header_name: header_name.to_string() });
                }
            }
        }

        Ok(greq_header)
    }
}

impl EnrichWith for GreqHeader {
    // Enrich values from another_object into self
    // Used to fill in missing values in self with values from another_object
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
    where
        Self: Sized,
    {
        // Override values in self with values from another_object if they are not empty
        if self.project.is_none() && object_to_merge.project.is_some() {
            self.project = object_to_merge.project.clone();
        }
        if self.output_folder.is_none() && object_to_merge.output_folder.is_some() {
            self.output_folder = object_to_merge.output_folder.clone();
        }
        if self.output_file_name.is_none() && object_to_merge.output_file_name.is_some() {
            self.output_file_name = object_to_merge.output_file_name.clone();
        }

        // Set base_request if not set in self
        if self.base_request.is_none() && object_to_merge.base_request.is_some() {
            self.base_request = object_to_merge.base_request.clone(); // Option can use clone
        }

        // Set depends_on if not set in self
        if self.depends_on.is_none() && object_to_merge.depends_on.is_some() {
            self.depends_on = object_to_merge.depends_on.clone(); // Option can use clone
        }

        Ok(())
    }
}

