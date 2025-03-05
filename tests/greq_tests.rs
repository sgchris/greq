use std::str::FromStr;
use greq::greq_object::greq::Greq;
use greq::greq_object::greq::GreqErrorCodes;

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
        greq.header().original_string,
        "output-folder: /path/to/output/folder\r\nproject: greq test"
    );
    assert_eq!(
        greq.content().original_string,
        "GET /some-url\r\nhost: greq-test.example.com"
    );
    assert_eq!(
        greq.footer().original_string,
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
    assert_eq!(parse_result.unwrap_err().code, GreqErrorCodes::TooFewSections);
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
        greq.header().original_string,
        "output-folder: /path/to/output/folder"
    ); // Header is empty
    assert_eq!(greq.content().original_string, "GET /some-url"); // Content is valid
    assert_eq!(
        greq.footer().original_string,
        "status-code: 200\r\nresponse-content contains: greq test result"
    ); // Footer is valid
} 