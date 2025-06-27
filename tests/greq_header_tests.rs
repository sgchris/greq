use greq::greq_object::greq_header::{GreqHeader, GreqHeaderError};

#[test]
fn test_parse_empty_header() {
    let result = GreqHeader::parse(&vec![]);
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header, GreqHeader::default());
}

#[test]
fn test_parse_empty_values() {
    let lines = vec![
        "project:",
        "depends-on:",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_err());
    
    // Should error on the first empty value encountered
    match result.unwrap_err() {
        GreqHeaderError::HeaderHasNoValue { header_name } => {
            assert_eq!(header_name, "project");
        }
        _ => panic!("Expected EmptyValue error"),
    }
}

#[test]
fn test_parse_colon_at_beginning() {
    let line_without_name = ":value_without_header_name";
    let lines = vec![line_without_name];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_err());

    match result.unwrap_err() {
        GreqHeaderError::HeaderHasNoName { line } => {
            assert_eq!(line, line_without_name);
        }
        _ => panic!("Expected EmptyValue error"),
    }
}

#[test]
fn test_parse_valid_complete_header() {
    let lines = vec![
        "project: MyProject",
        "depends-on: auth_request",
        "base-request: base.greq",
        "is-http: true",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, Some("MyProject".to_string()));
    assert_eq!(header.depends_on, Some("auth_request".to_string()));
    assert_eq!(header.base_request, Some("base.greq".to_string()));
    assert_eq!(header.is_http, true);
}

#[test]
fn test_parse_with_whitespace() {
    let lines = vec![
        "  project  :  MyProject  ",
        "   ",
        "\t\t",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, Some("MyProject".to_string()));
}

#[test]
fn test_parse_case_insensitive_headers() {
    let lines = vec![
        "PROJECT: MyProject",
        "DEPENDS-ON: auth",
        "Base-Request: base.greq",
        "IS-HTTP:true",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, Some("MyProject".to_string()));
    assert_eq!(header.depends_on, Some("auth".to_string()));
    assert_eq!(header.base_request, Some("base.greq".to_string()));
    assert_eq!(header.is_http, true);
}

#[test]
fn test_parse_missing_colon() {
    let lines = vec![
        "project: MyProject",
        "invalid_line_without_colon",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_err());

    match result.unwrap_err() {
        GreqHeaderError::LineHasNoColonSign { line } => {
            assert_eq!(line, "invalid_line_without_colon");
        }
        _ => panic!("Expected LineHasNoColonSign error"),
    }
}

#[test]
fn test_parse_unknown_header() {
    let lines = vec![
        "project: MyProject",
        "unknown-header: some_value",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_err());

    match result.unwrap_err() {
        GreqHeaderError::UnknownHeader { header_name } => {
            assert_eq!(header_name, "unknown-header");
        }
        _ => panic!("Expected UnknownHeader error"),
    }
}

#[test]
fn test_parse_multiple_colons() {
    let lines = vec![
        "project: http://example.com:8080/path",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, Some("http://example.com:8080/path".to_string()));
}

#[test]
fn test_parse_is_http_variations() {
    let test_cases = vec![
        ("is-http: false", false),
        ("is-http: true", true),
        ("is-http: yes", true),
        ("is-http: no", false),
        ("is-http: 1", true),
        ("is-http: 0", false),
        ("is-http: false", false),
        ("is-http: true", true),
    ];

    for (line, expected) in test_cases {
        let result = GreqHeader::parse(&vec![line]);
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.is_http, expected, "Failed for line: {}", line);
    }

    // check for invalid values
    let invalid_lines = vec![
        "is-http: maybe",
        "is-http: 2",
        "is-http: unknown",
    ];
    for line in invalid_lines {
        let result = GreqHeader::parse(&vec![line]);
        assert!(result.is_err());

        match result.unwrap_err() {
            GreqHeaderError::InvalidHeaderValue { line: err_line } => {
                assert_eq!(err_line, line);
            }
            _ => panic!("Expected InvalidHeaderValue error for line: {}", line),
        }
    }
}

#[test]
fn test_parse_duplicate_headers() {
    let lines = vec![
        "project: FirstProject",
        "project: SecondProject",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    // Should use the last value
    assert_eq!(header.project, Some("SecondProject".to_string()));
}

#[test]
fn test_parse_special_characters() {
    let lines = vec![
        "project: My-Project_123!",
        "depends-on: request:with:colons",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, Some("My-Project_123!".to_string()));
    assert_eq!(header.depends_on, Some("request:with:colons".to_string()));
}

#[test]
fn test_parse_unicode_characters() {
    let lines = vec![
        "project: プロジェクト名",
        "depends-on: αβγ_request",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, Some("プロジェクト名".to_string()));
    assert_eq!(header.depends_on, Some("αβγ_request".to_string()));
}

#[test]
fn test_parse_only_whitespace_lines() {
    let lines = vec![
        "   ",
        "\t\t\t",
        "\n",
        "     \t   ",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header, GreqHeader::default());
}

#[test]
fn test_parse_very_long_values() {
    let long_value = "a".repeat(1000);
    let formatted_line = format!("project: {}", long_value);
    let lines = vec![formatted_line.as_str()];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header.project, Some(long_value));
}

