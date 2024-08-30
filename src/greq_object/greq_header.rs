#![allow(dead_code, unused_variables)]

#[derive(Debug, Default)]
struct GreqHeader {
    original_string: String,

    project: String,          // the name of the project. Will be implemented.
    output_folder: String,    // path to a destination folder. Default current.
    output_file_name: String, // output filename. default current file name with ".response" extension.

    certificate: String, // Absolute path to the certificate file (pfx) certificate password will be handled later
    base_request: String,
}

impl GreqHeader {
    pub fn from_string(contents: &str) -> Result<GreqHeader, String> {
        // validate the contents
        GreqHeader::is_valid(contents)?;

        // initialize the object
        let mut greq_header = GreqHeader {
            original_string: contents.to_string(),
            ..Default::default()
        };

        let mut unknown_headers: Vec<&str> = vec![];
        // parse lines and assign properties
        contents.split("\r\n").for_each(|line| {
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
                _ => {
                    unknown_headers.push(header_name);
                }
            }
        });

        if !unknown_headers.is_empty() {
            // Return an error with the formatted message
            return Err(format!("Unknown headers: {}", unknown_headers.join(", ")));
        }

        Ok(greq_header)
    }

    // should be public as it's used as static
    pub fn is_valid(contents: &str) -> Result<bool, String> {
        // empty contents allowed
        if contents.trim().is_empty() {
            return Ok(true);
        }

        let mut errors: Vec<&str> = vec![];
        contents.split("\r\n").for_each(|line| {
            // must not be empty
            if line.is_empty() {
                errors.push("empty line in the header");
            }
            if !line.contains(":") {
                errors.push("the line in the header does not contain ':' character");
            }
        });

        return match errors.is_empty() {
            true => Ok(true),
            false => Err(errors.join(", ")),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_test_success() {
        // arrange
        let test_content =
            "output-folder: /some/folder\r\npfx-certificate: c:\\some\\folder\\cert.pfx";

        // act
        let result: GreqHeader = GreqHeader::from_string(test_content)
            .expect("Could not parse the test contents string");
        println!("greq_header: {:?}", result);

        assert_eq!(result.original_string, test_content);
        assert_eq!(result.output_folder, "/some/folder");
        assert_eq!(result.certificate, "c:\\some\\folder\\cert.pfx");
    }
}
