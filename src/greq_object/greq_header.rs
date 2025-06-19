use serde::{Deserialize, Serialize};
use crate::greq_object::traits::enrich_with_trait::EnrichWith;
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum GreqHeaderError {
    #[error("Unknown header '{header_name}' encountered in GreqHeader parsing")]
    UnknownHeader { header_name: String },
    #[error("The line '{line}' does not contain a colon sign")]
    LineHasNoColonSign { line: String },
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct GreqHeader {
    pub original_string: String,

    // the delimiter character to separate sections. Default '='.
    // (This header is used by the parent Greq object to parse the file.)
    pub delimiter: char,

    pub project: String,          // the name of the project. Will be implemented.
    pub output_folder: String,    // path to a destination folder. Default current.
    pub output_file_name: String, // output filename. default current file name with ".response" extension.

    // http and not https request
    pub is_http: Option<bool>,  

    // the request that this file extends
    pub base_request: Option<String>,

    // get_response that request before executing this one
    pub depends_on: Option<String>,
}

impl GreqHeader {
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

            // check if the line contains a colon
            if !line.contains(':') {
                return Err(GreqHeaderError::LineHasNoColonSign { line: line.to_string() });
            }

            let (header_name, header_value) = line.split_once(":").unwrap();
            let header_name = header_name.trim();
            let header_value = header_value.trim();

            match header_name.to_lowercase().as_str() {
                "project" => {
                    greq_header.project = header_value.to_string();
                }
                "depends-on" => {
                    greq_header.depends_on = Some(header_value.to_string());
                }
                "base-request" => {
                    greq_header.base_request = Some(header_value.to_string());
                }
                "output-folder" => {
                    greq_header.output_folder = header_value.to_string();
                }
                "output-file-name" => {
                    greq_header.output_file_name = header_value.to_string();
                }
                "is-http" => {
                    greq_header.is_http = Some(true);
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

    // Merge values from object_to_merge into self
    // Used to merge the header with the base request
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
    where
        Self: Sized
    {
        // Override values in self with values from object_to_merge if they are not empty
        if self.project.is_empty() && !object_to_merge.project.is_empty() {
            self.project = object_to_merge.project.to_string();
        }
        if self.output_folder.is_empty() && !object_to_merge.output_folder.is_empty() {
            self.output_folder = object_to_merge.output_folder.to_string();
        }
        if self.output_file_name.is_empty() && !object_to_merge.output_file_name.is_empty() {
            self.output_file_name = object_to_merge.output_file_name.to_string();
        }

        // Set is_http if not set in self
        if self.is_http.is_none() {
            if let Some(is_http_value) = object_to_merge.is_http {
                self.is_http = Some(is_http_value);
            }
        }

        // Set base_request if not set in self
        if self.base_request.is_none() {
            if object_to_merge.base_request.is_some() {
                self.base_request = object_to_merge.base_request.clone(); // Option can use clone
            }
        }

        // Set depends_on if not set in self
        if self.depends_on.is_none() {
            if object_to_merge.depends_on.is_some() {
                self.depends_on = object_to_merge.depends_on.clone(); // Option can use clone
            }
        }

        Ok(())
    }
}

