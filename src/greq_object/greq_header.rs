use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::greq_object::traits::enrich_with_trait::EnrichWith;
use thiserror::Error;

#[derive(Serialize, Deserialize, Default, Debug)]
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

#[derive(Debug, PartialEq, Error)]
pub enum GreqHeaderError {
    #[error("Unknown header encountered in GreqHeader parsing")]
    UnknownHeader,
    #[error("The line does not contain a colon sign")]
    LineHasNoColonSign,
}

impl FromStr for GreqHeader {
    type Err = GreqHeaderError;

    fn from_str(contents: &str) -> Result<GreqHeader, Self::Err> {
        // validate the contents
        GreqHeader::is_valid(contents)?;

        // initialize the object
        let mut greq_header = GreqHeader {
            original_string: contents.to_string(),
            ..Default::default()
        };

        // parse lines and assign properties
        contents.lines().try_for_each(|line| {
            let line_parts: Vec<&str> = line.split(":").collect();
            let header_name: &str = line_parts[0].trim();
            let header_value = line_parts[1..].join(":").trim().to_lowercase().to_string();

            match header_name.to_lowercase().as_str() {
                "project" => {
                    greq_header.project = header_value;
                }
                "depends-on" => {
                    greq_header.depends_on = Some(header_value);
                }
                "base-request" => {
                    greq_header.base_request = Some(header_value);
                }
                "output-folder" => {
                    greq_header.output_folder = header_value;
                }
                "output-file-name" => {
                    greq_header.output_file_name = header_value;
                }
                "is-http" => {
                    greq_header.is_http = Some(true);
                }
                _ => {
                    return Err(GreqHeaderError::UnknownHeader)
                }
            }

            Ok(())
        })?;

        Ok(greq_header)
    }
}

impl EnrichWith for GreqHeader {
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

impl GreqHeader {
    // should be public as it's used as static
    pub fn is_valid(contents: &str) -> Result<bool, GreqHeaderError> {
        // empty contents allowed
        if contents.trim().is_empty() {
            return Ok(true);
        }

        if contents.lines().find(|line| !line.trim().contains(":")).is_some() {
            return Err(GreqHeaderError::LineHasNoColonSign);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_input() {
        let input = "project: MyProject\n\
                     base-request: GET /api/data\n\
                     output-folder: /tmp\n\
                     output-file-name: response.json";
        let result = GreqHeader::from_str(input);
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.project, "myproject");
        assert_eq!(header.base_request.unwrap_or("".to_string()), "get /api/data");
        assert_eq!(header.output_folder, "/tmp");
        assert_eq!(header.output_file_name, "response.json");
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let result = GreqHeader::from_str(input);
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.original_string, "");
    }

    #[test]
    fn test_invalid_format_missing_colon() {
        let input = "project MyProject\n\
                     base-request: GET /api/data";
        let result = GreqHeader::from_str(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, GreqHeaderErrorCodes::LineHasNoColonSign);
    }

    #[test]
    fn test_unknown_headers() {
        let input = "project: MyProject\n\
                     unknown-header: some value\n\
                     base-request: GET /api/data";
        let result = GreqHeader::from_str(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, GreqHeaderErrorCodes::UnknownHeader);
    }

    #[test]
    fn test_multiple_headers() {
        let input = "project: MyProject\n\
                     output-folder: /tmp\n\
                     output-folder: /var/tmp\n\
                     base-request: GET /api/data";
        let result = GreqHeader::from_str(input);
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.output_folder, "/var/tmp"); // Last occurrence should overwrite previous
    }

    #[test]
    fn test_is_valid_empty() {
        let result = GreqHeader::is_valid("");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_is_valid_no_colon() {
        let input = "project MyProject\n\
                     base-request: GET /api/data";
        let result = GreqHeader::is_valid(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, GreqHeaderErrorCodes::LineHasNoColonSign);
    }

    #[test]
    fn test_enrich_with_empty_self() {
        let mut header = GreqHeader {
            project: String::new(),
            output_folder: String::new(),
            output_file_name: String::new(),
            is_http: None,
            base_request: None,
            depends_on: None,
            ..Default::default()
        };

        let object_to_merge = GreqHeader {
            project: String::from("MyProject"),
            output_folder: String::from("/output/folder"),
            output_file_name: String::from("output.txt"),
            is_http: Some(true),
            base_request: Some(String::from("/path/to/base/request")),
            depends_on: Some(String::from("/path/to/dependson")),
            ..Default::default()
        };

        header.enrich_with(&object_to_merge).unwrap();

        assert_eq!(header.project, "MyProject");
        assert_eq!(header.output_folder, "/output/folder");
        assert_eq!(header.output_file_name, "output.txt");
        assert_eq!(header.is_http, Some(true));
        assert_eq!(header.base_request, Some(String::from("/path/to/base/request")));
        assert_eq!(header.depends_on, Some(String::from("/path/to/dependson")));
    }

    #[test]
    fn test_enrich_with_partial_values() {
        let mut header = GreqHeader {
            project: String::from("ExistingProject"),
            output_folder: String::new(),
            output_file_name: String::from("existing_output.txt"),
            is_http: Some(false),
            base_request: None,
            depends_on: Some(String::from("/path/to/depends/on")),
            ..Default::default()
        };

        let object_to_merge = GreqHeader {
            project: String::from("NewProject"),
            output_folder: String::from("/new/output/folder"),
            output_file_name: String::from("new_output.txt"),
            is_http: Some(true),
            base_request: Some(String::from("/new/path/to/base/request")),
            depends_on: Some(String::from("/new/path/to/depends/on")),
            ..Default::default()
        };

        header.enrich_with(&object_to_merge).unwrap();

        assert_eq!(header.project, "ExistingProject"); // Not overridden
        assert_eq!(header.output_folder, "/new/output/folder");
        assert_eq!(header.output_file_name, "existing_output.txt"); // Not overridden
        assert_eq!(header.is_http, Some(false)); // Not overridden
        assert_eq!(header.base_request, Some(String::from("/new/path/to/base/request")));
        assert_eq!(header.depends_on, Some(String::from("/path/to/depends/on"))); // Not overridden
    }

    #[test]
    fn test_enrich_with_no_changes() {
        let mut header = GreqHeader {
            project: String::from("ExistingProject"),
            output_folder: String::from("/existing/output/folder"),
            output_file_name: String::from("existing_output.txt"),
            is_http: Some(false),
            base_request: Some(String::from("/existing/path/to/base/request")),
            depends_on: Some(String::from("/existing/path/to/depends/on")),
            ..Default::default()
        };

        let object_to_merge = GreqHeader {
            project: String::from("NewProject"),
            output_folder: String::from("/new/output/folder"),
            output_file_name: String::from("new_output.txt"),
            is_http: Some(true),
            base_request: Some(String::from("/new/path/to/base/request")),
            depends_on: Some(String::from("/new/path/to/depends/on")),
            ..Default::default()
        };

        header.enrich_with(&object_to_merge).unwrap();

        // No fields should be changed
        assert_eq!(header.project, "ExistingProject");
        assert_eq!(header.output_folder, "/existing/output/folder");
        assert_eq!(header.output_file_name, "existing_output.txt");
        assert_eq!(header.is_http, Some(false));
        assert_eq!(header.base_request, Some(String::from("/existing/path/to/base/request")));
        assert_eq!(header.depends_on, Some(String::from("/existing/path/to/depends/on")));
    }
}
