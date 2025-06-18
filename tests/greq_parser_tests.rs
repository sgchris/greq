use greq::constants::{DEFAULT_DELIMITER_CHAR, DELIMITER_MIN_LENGTH};
use greq::greq_object::greq::GreqErrorCodes;
use greq::greq_object::greq_parser::{extract_delimiter, parse_sections};


#[test]
fn test_extract_delimiter_valid_cases() {
    // Test with valid delimiter specification
    assert_eq!(extract_delimiter("delimiter: #"), '#');
    assert_eq!(extract_delimiter("Delimiter: @"), '@');
    assert_eq!(extract_delimiter("DELIMITER: *"), '*');

    // Test with whitespace around delimiter
    assert_eq!(extract_delimiter("delimiter:   #   "), '#');
    assert_eq!(extract_delimiter("delimiter:\t@\t"), '@');

    // Test with delimiter in middle of content
    let content = "some content\ndelimiter: %\nmore content";
    assert_eq!(extract_delimiter(content), '%');
}

#[test]
fn test_extract_delimiter_edge_cases() {
    // Test with no delimiter specification
    assert_eq!(extract_delimiter("no delimiter here"), DEFAULT_DELIMITER_CHAR);
    assert_eq!(extract_delimiter(""), DEFAULT_DELIMITER_CHAR);

    // Test with empty value after colon
    assert_eq!(extract_delimiter("delimiter:"), DEFAULT_DELIMITER_CHAR);
    assert_eq!(extract_delimiter("delimiter:   "), DEFAULT_DELIMITER_CHAR);

    // Test with no colon
    assert_eq!(extract_delimiter("delimiter"), DEFAULT_DELIMITER_CHAR);

    // Test with multiple delimiters (should take first)
    assert_eq!(extract_delimiter("delimiter: #@"), '#');

    // Test case insensitive matching
    assert_eq!(extract_delimiter("dElImItEr: ^"), '^');

    // Test with delimiter not at start of line
    assert_eq!(extract_delimiter("  delimiter: &"), DEFAULT_DELIMITER_CHAR);

    // Test with multiple delimiter lines (should take first)
    let content = "delimiter: #\ndelimiter: @";
    assert_eq!(extract_delimiter(content), '#');
}

#[test]
fn test_parse_sections_valid_cases() {
    // Test basic three-section parsing
    let delimiter_str = "#".repeat(DELIMITER_MIN_LENGTH);
    let content = format!("section1 line1\nsection1 line2\n{}\nsection2 line1\n{}\nsection3 line1", delimiter_str, delimiter_str);
    let result = parse_sections(&content, '#').unwrap();

    assert_eq!(result[0], vec!["section1 line1", "section1 line2"]);
    assert_eq!(result[1], vec!["section2 line1"]);
    assert_eq!(result[2], vec!["section3 line1"]);

    // Test with different delimiter
    let content = format!("first\n{}\nsecond\n{}\nthird", "@".repeat(DELIMITER_MIN_LENGTH), "@".repeat(DELIMITER_MIN_LENGTH));
    let result = parse_sections(&content, '@').unwrap();

    assert_eq!(result[0], vec!["first"]);
    assert_eq!(result[1], vec!["second"]);
    assert_eq!(result[2], vec!["third"]);
}

#[test]
fn test_parse_sections_empty_sections() {
    let delimiter_str = "#".repeat(DELIMITER_MIN_LENGTH);

    // Test with empty first section
    let content = format!("{}\nsection2\n{}\nsection3", delimiter_str, delimiter_str);
    let result = parse_sections(&content, '#').unwrap();
    assert_eq!(result[0], Vec::<&str>::new());
    assert_eq!(result[1], vec!["section2"]);
    assert_eq!(result[2], vec!["section3"]);

    // Test with empty middle section
    let content = format!("section1\n{}\n{}\nsection3", delimiter_str, delimiter_str);
    let result = parse_sections(&content, '#').unwrap();
    assert_eq!(result[0], vec!["section1"]);
    assert_eq!(result[1], Vec::<&str>::new());
    assert_eq!(result[2], vec!["section3"]);

    // Test with empty last section
    let content = format!("section1\n{}\nsection2\n{}", delimiter_str, delimiter_str);
    let result = parse_sections(&content, '#').unwrap();
    assert_eq!(result[0], vec!["section1"]);
    assert_eq!(result[1], vec!["section2"]);
    assert_eq!(result[2], Vec::<&str>::new());

    // Test with all sections empty
    let content = format!("{}\n{}", delimiter_str, delimiter_str);
    let result = parse_sections(&content, '#').unwrap();
    assert_eq!(result[0], Vec::<&str>::new());
    assert_eq!(result[1], Vec::<&str>::new());
    assert_eq!(result[2], Vec::<&str>::new());
}



#[test]
fn test_parse_sections_whitespace_handling() {
    let delimiter_str = DEFAULT_DELIMITER_CHAR.to_string().repeat(DELIMITER_MIN_LENGTH);

    // Test with blank lines
    let content = format!("line1\n\nline2\n{}\n\nsection2\n\n{}\nsection3\n", delimiter_str, delimiter_str);

    let result = match parse_sections(&content, DEFAULT_DELIMITER_CHAR) {
        Ok(sections) => sections,
        Err(e) => panic!("Failed to parse sections: {:?}", e),
    }; 

    assert_eq!(result[0], vec!["line1", "", "line2"]);
    assert_eq!(result[1], vec!["", "section2", ""]);
    assert_eq!(result[2], vec!["section3"]);

    // Test with only whitespace lines
    let content = format!("   \n\t\n{}\n  \n{}\n\t  ", delimiter_str, delimiter_str);
    let result = parse_sections(&content, DEFAULT_DELIMITER_CHAR).unwrap();
    assert_eq!(result[0], vec!["   ", "\t"]);
    assert_eq!(result[1], vec!["  "]);
    assert_eq!(result[2], vec!["\t  "]);
}

#[test]
fn test_parse_sections_delimiter_variations() {
    // Test with delimiter longer than minimum
    let long_delimiter = "#".repeat(DELIMITER_MIN_LENGTH + 5);
    let content = format!("section1\n{}\nsection2\n{}\nsection3", long_delimiter, long_delimiter);
    let result = parse_sections(&content, '#').unwrap();
    assert_eq!(result[0], vec!["section1"]);
    assert_eq!(result[1], vec!["section2"]);
    assert_eq!(result[2], vec!["section3"]);

    // Test with delimiter that has extra characters after
    let delimiter_with_extra = format!("{}extra text", "#".repeat(DELIMITER_MIN_LENGTH));
    let content = format!("section1\n{}\nsection2\n{}\nsection3", delimiter_with_extra, delimiter_with_extra);
    let result = parse_sections(&content, '#').unwrap();
    assert_eq!(result[0], vec!["section1"]);
    assert_eq!(result[1], vec!["section2"]);
    assert_eq!(result[2], vec!["section3"]);
}

#[test]
fn test_parse_sections_error_too_few_sections() {
    let delimiter_str = "#".repeat(DELIMITER_MIN_LENGTH);

    // Test with no delimiters
    let result = parse_sections("just one section", '#');
    assert_eq!(result, Err(GreqErrorCodes::TooFewSections));

    // Test with only one delimiter
    let content = format!("section1\n{}\nsection2", delimiter_str);
    let result = parse_sections(&content, '#');
    assert_eq!(result, Err(GreqErrorCodes::TooFewSections));

    // Test with empty content
    let result = parse_sections("", '#');
    assert_eq!(result, Err(GreqErrorCodes::TooFewSections));
}

#[test]
fn test_parse_sections_error_too_many_sections() {
    let delimiter_str = "#".repeat(DELIMITER_MIN_LENGTH);

    // Test with three delimiters (creating four sections)
    let content = format!("section1\n{}\nsection2\n{}\nsection3\n{}\nsection4", delimiter_str, delimiter_str, delimiter_str);
    let result = parse_sections(&content, '#');
    assert_eq!(result, Err(GreqErrorCodes::TooManySections));

    // Test with many delimiters
    let content = format!("s1\n{}\ns2\n{}\ns3\n{}\ns4\n{}\ns5", delimiter_str, delimiter_str, delimiter_str, delimiter_str);
    let result = parse_sections(&content, '#');
    assert_eq!(result, Err(GreqErrorCodes::TooManySections));
}

#[test]
fn test_parse_sections_special_characters() {
    // Test with various special characters as delimiters
    let special_chars = ['@', '*', '%', '&', '!', '~', '^'];

    for &char in &special_chars {
        let delimiter_str = char.to_string().repeat(DELIMITER_MIN_LENGTH);
        let content = format!("first\n{}\nsecond\n{}\nthird", delimiter_str, delimiter_str);
        let result = parse_sections(&content, char).unwrap();

        assert_eq!(result[0], vec!["first"]);
        assert_eq!(result[1], vec!["second"]);
        assert_eq!(result[2], vec!["third"]);
    }
}

#[test]
fn test_parse_sections_line_endings() {
    let delimiter_str = "#".repeat(DELIMITER_MIN_LENGTH);

    // Test with different line ending styles (should be handled by lines())
    let content_unix = format!("section1\n{}\nsection2\n{}\nsection3", delimiter_str, delimiter_str);
    let content_windows = content_unix.replace('\n', "\r\n");

    let result_unix = parse_sections(&content_unix, '#').unwrap();
    let result_windows = parse_sections(&content_windows, '#').unwrap();

    // Both should produce the same result
    assert_eq!(result_unix[0], vec!["section1"]);
    assert_eq!(result_unix[1], vec!["section2"]);
    assert_eq!(result_unix[2], vec!["section3"]);

    assert_eq!(result_windows[0], vec!["section1"]);
    assert_eq!(result_windows[1], vec!["section2"]);
    assert_eq!(result_windows[2], vec!["section3"]);
}

#[test]
fn test_parse_sections_edge_case_minimum_length() {
    // Test that delimiter must meet minimum length requirement
    let short_delimiter = "#".repeat(DELIMITER_MIN_LENGTH - 1);
    let proper_delimiter = "#".repeat(DELIMITER_MIN_LENGTH);

    // Short delimiter should not be recognized as delimiter
    let content = format!("section1\n{}\nsection2\n{}\nsection3", short_delimiter, proper_delimiter);
    let result = parse_sections(&content, '#');
    assert_eq!(result, Err(GreqErrorCodes::TooFewSections));
}

#[test]
fn test_integration_extract_and_parse() {
    // Test using extract_delimiter result with parse_sections
    let content = format!(
        "delimiter: @\nfirst section\n{}\nsecond section\n{}\nthird section",
        "@".repeat(DELIMITER_MIN_LENGTH),
        "@".repeat(DELIMITER_MIN_LENGTH)
    );

    let delimiter = extract_delimiter(&content);
    let result = parse_sections(&content, delimiter).unwrap();

    assert_eq!(result[0], vec!["delimiter: @", "first section"]);
    assert_eq!(result[1], vec!["second section"]);
    assert_eq!(result[2], vec!["third section"]);
}

