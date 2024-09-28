use crate::greq_object::from_string_trait::FromString;

#[derive(Debug, Default)]
pub struct GreqHeader {
    pub original_string: String,

    pub project: String,          // the name of the project. Will be implemented.
    pub output_folder: String,    // path to a destination folder. Default current.
    pub output_file_name: String, // output filename. default current file name with ".response" extension.

    // absolute path to the certificate (PFX)
    pub certificate: String, // Absolute path to the certificate file (pfx) certificate password will be handled later
    // http and not https request
    pub is_http: bool,

    // the request that this file extends
    pub base_request: String,
    // execute that request before executing this one
    pub depends_on: String,
}

impl FromString for GreqHeader {
    fn from_string(contents: &str) -> Result<GreqHeader, String> {
        // validate the contents
        GreqHeader::is_valid(contents)?;

        // initialize the object
        let mut greq_header = GreqHeader {
            original_string: contents.to_string(),
            ..Default::default()
        };

        let mut unknown_headers: Vec<&str> = vec![];
        // parse lines and assign properties
        contents.lines().for_each(|line| {
            let line_parts: Vec<&str> = line.split(":").collect();
            let header_name: &str = line_parts[0].trim();
            let header_value = line_parts[1..].join(":").trim().to_lowercase().to_string();

            match header_name.to_lowercase().as_str() {
                "project" => {
                    greq_header.project = header_value;
                }
                "base-request" => {
                    greq_header.base_request = header_value;
                }
                "output-folder" => {
                    greq_header.output_folder = header_value;
                }
                "output-file-name" => {
                    greq_header.output_file_name = header_value;
                }
                "certificate" => {
                    greq_header.certificate = header_value;
                }
                "is-http" => {
                    greq_header.is_http = true;
                }
                _ => {
                    unknown_headers.push(header_name);
                }
            }
        });

        if !unknown_headers.is_empty() {
            // Return an error with the formatted message
            return Err(format!("unknown headers: {}", unknown_headers.join(", ")));
        }

        Ok(greq_header)
    }
}

impl GreqHeader {
    // should be public as it's used as static
    pub fn is_valid(contents: &str) -> Result<bool, String> {
        // empty contents allowed
        if contents.trim().is_empty() {
            return Ok(true);
        }

        if contents.lines().find(|line| line.trim().is_empty()).is_some() {
            return Err("empty line in the header".to_string());
        }

        if contents.lines().find(|line| !line.trim().contains(":")).is_some() {
            return Err("line without ':' character".to_string());
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
                     output-file-name: response.json\n\
                     certificate: /path/to/certificate.pfx";
        let result = GreqHeader::from_string(input);
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.project, "myproject");
        assert_eq!(header.base_request, "get /api/data");
        assert_eq!(header.output_folder, "/tmp");
        assert_eq!(header.output_file_name, "response.json");
        assert_eq!(header.certificate, "/path/to/certificate.pfx");
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let result = GreqHeader::from_string(input);
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.original_string, "");
    }

    #[test]
    fn test_invalid_format_missing_colon() {
        let input = "project MyProject\n\
                     base-request: GET /api/data";
        let result = GreqHeader::from_string(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "line without ':' character");
    }

    #[test]
    fn test_invalid_format_empty_line() {
        let input = "project: MyProject\n\
                     \n\
                     base-request: GET /api/data";
        let result = GreqHeader::from_string(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "empty line in the header");
    }

    #[test]
    fn test_unknown_headers() {
        let input = "project: MyProject\n\
                     unknown-header: some value\n\
                     base-request: GET /api/data";
        let result = GreqHeader::from_string(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown headers: unknown-header"));
    }

    #[test]
    fn test_multiple_headers() {
        let input = "project: MyProject\n\
                     output-folder: /tmp\n\
                     output-folder: /var/tmp\n\
                     base-request: GET /api/data\n\
                     certificate: /path/to/certificate.pfx";
        let result = GreqHeader::from_string(input);
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
    fn test_is_valid_with_empty_lines() {
        let input = "project: MyProject\n\
                     \n\
                     base-request: GET /api/data";
        let result = GreqHeader::is_valid(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty line in the header"));
    }

    #[test]
    fn test_is_valid_no_colon() {
        let input = "project MyProject\n\
                     base-request: GET /api/data";
        let result = GreqHeader::is_valid(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("line without ':' character"));
    }
}
