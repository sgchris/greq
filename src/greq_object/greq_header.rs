use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::greq_object::{
    greq_response::GreqResponse, 
    traits::enrich_with_trait::EnrichWith
};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::io; 
use std::env;

#[derive(Debug, PartialEq, Error)]
pub enum GreqHeaderError {
    #[error("Unknown property '{header_name}' in the header section")]
    UnknownHeader { header_name: String },
    #[error("The line '{line}' does not contain a colon sign")]
    LineHasNoColonSign { line: String },
    #[error("No property name before the colon sign: '{line}'")]
    HeaderHasNoName { line: String },
    #[error("No value after the colon sign for '{header_name}'")]
    HeaderHasNoValue { header_name: String },
    #[error("Not valid line in the header section: '{line}'")]
    InvalidHeaderValue { line: String },
    #[error("The file does not exist: '{path}'")]
    FileDoesNotExist { path: String },
    #[error("Error: '{error}'")]
    GeneralError { error: String },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct GreqHeader {
    // absolute path of the Greq file
    pub absolute_path: String,

    // the delimiter character to separate sections. Default '='.
    // (This header is used by the parent Greq object to parse the file.)
    pub delimiter: char,

    pub project: Option<String>,          // the name of the project. Will be implemented.

    // http and not https request
    pub is_http: bool,  

    // the request that this file extends
    pub extends: Option<String>,

    // get_response that request before executing this one
    pub depends_on: Option<String>,
}

// override some default values
impl Default for GreqHeader {
    fn default() -> Self {
        GreqHeader {
            absolute_path: String::new(),
            delimiter: '=',
            is_http: false,
            project: None,
            extends: None,
            depends_on: None,
        }
    }
}

impl GreqHeader {

    /// Parse the header lines and return a GreqHeader object.
    pub fn parse(
        header_lines: &Vec<&str>,
        greq_file_path: String,
        base_request: Option<&GreqHeader>,
        dependency_response: Option<&GreqResponse>,
    ) -> Result<GreqHeader, GreqHeaderError> {

        // convert header_lines to COW (changeable on write) strings
        let mut cow_header_lines = GreqHeader::strs_to_cows(header_lines);

        // replace placeholders in the header lines with values from the dependency response
        if let Some(dependency_response_obj) = dependency_response {
            GreqHeader::replace_placeholders_in_lines(&mut cow_header_lines, dependency_response_obj);
        }

        // parse the header lines and initialize the GreqHeader object
        let mut greq_header = GreqHeader::parse_lines_into_greq_header_object(&cow_header_lines)?;
        greq_header.absolute_path = greq_file_path;

        // enrich with base_request 
        // or check if base request property provided and check if it exists
        GreqHeader::enrich_with_base_request_or_check_if_provided(&mut greq_header, base_request)?;

        // After the header was enriched with the base request,
        // replace placeholders in the header lines with values from the dependency response
        if dependency_response.is_some() {
            if base_request.is_some() {
                let dependency_response_obj = dependency_response.unwrap();
                GreqHeader::replace_placeholders_in_lines(&mut cow_header_lines, dependency_response_obj);
            }
        } else if greq_header.depends_on.is_some() {
            GreqHeader::check_and_update_depends_on(&mut greq_header);
        }

        Ok(greq_header)
    }

    /// parse the lines that are related to the header of the request
    fn parse_lines_into_greq_header_object(header_lines: &Vec<Cow<str>>) -> Result<GreqHeader, GreqHeaderError> {
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
                "is-http" => {
                    greq_header.is_http = match header_value.to_lowercase().as_str() {
                        "true" | "yes" | "1" => true,
                        "false" | "no" | "0" => false,
                        _ => return Err(GreqHeaderError::InvalidHeaderValue { line: line.to_string() }),
                    };
                }
                "base-request" => {
                    greq_header.extends = Some(header_value.to_string());
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

    /// Enrich the GreqHeader object with the base request or check if it is provided.
    /// If the base request is provided, check that the file exists
    fn enrich_with_base_request_or_check_if_provided(
        greq_header: &mut GreqHeader,
        base_request: Option<&GreqHeader>,
    ) -> Result<(), GreqHeaderError> {
        // if base_request is provided, enrich the greq_header with it
        if let Some(base_request) = base_request {
            greq_header.enrich_with(base_request).map_err(|e| GreqHeaderError::GeneralError { error: e })?;
        } else if greq_header.extends.is_some() {
            // resolve the base_request file path
            let mut base_request_name = Cow::from(greq_header.extends.as_ref().unwrap());
            if !base_request_name.ends_with(".greq") {
                // if the base_request_name does not end with .greq, add it
                base_request_name.to_mut().push_str(".greq");
            }

            // get the absolute path of the base request file
            let absolute_base_path = GreqHeader::resolve_and_check_file_exists(&base_request_name, Some(greq_header.absolute_path.as_ref()))
                .map_err(|_| GreqHeaderError::FileDoesNotExist { path: base_request_name.to_string() })?;

            // update the base_request field with the absolute path
            greq_header.extends = Some(absolute_base_path);
        }

        Ok(())
    }

    /// Check if the depends_on property is set, and if so, check if the file exists and update the
    /// value to the absolute path.
    fn check_and_update_depends_on(greq_header: &mut GreqHeader) -> Result<(), GreqHeaderError> {
        // if the depends_on property is set, check if the file exists
        let absolute_depends_on_path = GreqHeader::resolve_and_check_file_exists(
            greq_header.depends_on.as_ref().unwrap(),
            Some(&greq_header.absolute_path),
        ).map_err(|_| GreqHeaderError::FileDoesNotExist { path: greq_header.depends_on.as_ref().unwrap().to_string() })?;

        greq_header.depends_on = Some(absolute_depends_on_path);

        Ok(())
    }

    /// check if a file exists, and return its absolute path.
    fn resolve_and_check_file_exists(
        file_path: &str, // Changed to &str
        base_path: Option<&str>, // Changed to Option<&str>
    ) -> io::Result<String> {
        let candidate_path = if Path::new(file_path).is_absolute() { // Use Path::new for check
            // If the provided path is already absolute, use it directly
            PathBuf::from(file_path) // Convert &str to PathBuf
        } else {
            // If the provided path is relative, resolve it against the base_path
            // or the current working directory if base_path is None.
            let actual_base = match base_path {
                Some(path_str) => PathBuf::from(path_str), // Convert &str to PathBuf
                None => env::current_dir()?, // Get current working directory, propagate error if any
            };
            actual_base.join(file_path) // Join PathBuf with &str
        };

        // Check if the candidate path exists
        if candidate_path.exists() {
            // If it exists, return its canonicalized absolute path as a String.
            // canonicalize() resolves all symlinks, `.` and `..` components.
            candidate_path.canonicalize()? // This returns Result<PathBuf, io::Error>
                .to_str() // Convert Path to &str (Option<&str>)
                .map(|s| s.to_owned()) // Convert &str to String (Option<String>)
                .ok_or_else(|| { // If None (path is not valid UTF-8), return an error
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Resolved path contains invalid Unicode: {}", candidate_path.display()),
                    )
                })
        } else {
            // If the file does not exist at the candidate path, return a NotFound error.
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", candidate_path.display()),
            ))
        }
    }

    /// convert header_lines to COW (changeable on write) strings
    #[inline(always)]
    fn strs_to_cows<'a>(strs: &'a Vec<&'a str>) -> Vec<Cow<'a, str>> {
        strs.iter()
            .map(|&s| Cow::from(s))
            .collect()
    }

    /// Replace placeholders in the header lines with values from 
    /// get_var in the GreqResponse object.
    ///
    /// TODO: Check if it's possible to make less "clone" calls
    pub fn replace_placeholders_in_lines(
        header_lines: &mut Vec<Cow<str>>,
        greq_response: &GreqResponse,
    ) {
        // regex that finds "$(variable_name)" in the line, without escaping
        let re = regex::Regex::new(r"(?<!\\)\$\(([^)]+)\)").unwrap();

        // replace the placeholders in the header lines
        for line in header_lines.iter_mut() {
            if !re.is_match(line) {
                continue; // no placeholders to replace
            }

            // replace the placeholders in the line and change to owned COW
            *line = re.replace_all(line, |caps: &regex::Captures| {
                let var_name = &caps[1];
                greq_response.get_var(var_name)
            }).into_owned().into();
        }
    }
}

impl EnrichWith for GreqHeader {
    // Enrich values from another_object into self
    // Currently it only overrides the project and depends_on fields. All he other aren't optional.
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
    where
        Self: Sized,
    {
        // Override values in self with values from another_object if they are not empty
        if self.project.is_none() && object_to_merge.project.is_some() {
            self.project = object_to_merge.project.clone();
        }

        // Set depends_on if not set in self
        if self.depends_on.is_none() && object_to_merge.depends_on.is_some() {
            self.depends_on = object_to_merge.depends_on.clone(); // Option can use clone
        }

        Ok(())
    }
}

