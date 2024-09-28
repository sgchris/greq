use crate::greq_object::from_string_trait::FromString;
use crate::greq_object::greq_content::GreqContent;
use crate::greq_object::greq_footer::GreqFooter;
use crate::greq_object::greq_header::GreqHeader;

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

        if part_number != 2 {
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

        assert_eq!(greq.header.original_string, ""); // Header is empty
        assert_eq!(greq.content.original_string, "GET /some-url"); // Content is valid
        assert_eq!(greq.footer.original_string, "status-code: 200\r\nresponse-content contains: greq test result"); // Footer is valid
    }
}

