#![allow(dead_code, unused_variables)]

#[derive(Debug, Default)]
struct GreqHeader {
    original_string: String,

    // project_name: String, // the name of the project. Will be implemented.
    output_folder: String,    // path to a destination folder. Default current.
    output_file_name: String, // output filename. default current file name with ".response" extension.

    pfx_certificate: String, // Absolute path to the certificate file (pfx)
}

impl GreqHeader {
    pub fn from_string(contents: &str) -> Result<GreqHeader, &'static str> {
        if !GreqHeader::is_valid(contents) {
            return Err("The provided contents for the header are not valid");
        }

        // initialize the object
        let mut greq_header = GreqHeader::default();
        greq_header.original_string = contents.to_string();

        // parse lines and assign properties
        contents.split("\r\n").for_each(|line| {
            let line_parts: Vec<&str> = line.split(":").collect();
            let header_name: &str = line_parts[0].trim();
            let header_value = line_parts[1..].join(":").trim().to_lowercase().to_string();

            match header_name.to_lowercase().as_str() {
                "output-folder" => {
                    greq_header.output_folder = header_value;
                }
                "output-file-name" => {
                    greq_header.output_file_name = header_value;
                }
                "pfx-certificate" => {
                    greq_header.pfx_certificate = header_value;
                }
                _ => {}
            }
        });

        Ok(greq_header)
    }

    pub fn is_valid(contents: &str) -> bool {
        // empty contents allowed
        if contents.trim().len() == 0 {
            println!("empty contents of the header");
            return true;
        }

        let has_errors: bool = contents.split("\r\n").any(|line| {
            // must not be empty
            if line.is_empty() {
                println!("empty line in the header");
                true
            } else if !line.contains(":") {
                println!("the line in the header does not contain ':' character");
                true
            } else {
                false
            }
        });

        !has_errors
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
        assert_eq!(result.pfx_certificate, "c:\\some\\folder\\cert.pfx");
    }
}
