use greq::greq_object::greq_header::{GreqHeader, GreqHeaderError};

#[test]
fn test_parse_empty_header() {
    let result = GreqHeader::parse(&vec![]);
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header, GreqHeader::default());
}

#[test]
fn test_parse_valid_complete_header() {
    let lines = vec![
        "project: MyProject",
        "output-folder: /tmp/responses",
        "output-file-name: test.response",
        "depends-on: auth_request",
        "base-request: base.greq",
        "is-http:",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, "MyProject");
    assert_eq!(header.output_folder, "/tmp/responses");
    assert_eq!(header.output_file_name, "test.response");
    assert_eq!(header.depends_on, Some("auth_request".to_string()));
    assert_eq!(header.base_request, Some("base.greq".to_string()));
    assert_eq!(header.is_http, Some(true));
}

#[test]
fn test_parse_with_whitespace() {
    let lines = vec![
        "  project  :  MyProject  ",
        "\toutput-folder\t:\t/tmp/responses\t",
        " output-file-name : test.response ",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, "MyProject");
    assert_eq!(header.output_folder, "/tmp/responses");
    assert_eq!(header.output_file_name, "test.response");
}

#[test]
fn test_parse_with_empty_lines() {
    let lines = vec![
        "",
        "project: MyProject",
        "   ",
        "output-folder: /tmp",
        "\t\t",
        "",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, "MyProject");
    assert_eq!(header.output_folder, "/tmp");
}

#[test]
fn test_parse_case_insensitive_headers() {
    let lines = vec![
        "PROJECT: MyProject",
        "Output-Folder: /tmp",
        "DEPENDS-ON: auth",
        "Base-Request: base.greq",
        "IS-HTTP:",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, "MyProject");
    assert_eq!(header.output_folder, "/tmp");
    assert_eq!(header.depends_on, Some("auth".to_string()));
    assert_eq!(header.base_request, Some("base.greq".to_string()));
    assert_eq!(header.is_http, Some(true));
}

#[test]
fn test_parse_missing_colon() {
    let lines = vec![
        "project: MyProject",
        "invalid_line_without_colon",
        "output-folder: /tmp",
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
fn test_parse_empty_values() {
    let lines = vec![
        "project:",
        "output-folder:",
        "depends-on:",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, "");
    assert_eq!(header.output_folder, "");
    assert_eq!(header.depends_on, Some("".to_string()));
}

#[test]
fn test_parse_multiple_colons() {
    let lines = vec![
        "project: http://example.com:8080/path",
        "output-folder: C:\\Users\\test:folder",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, "http://example.com:8080/path");
    assert_eq!(header.output_folder, "C:\\Users\\test:folder");
}

#[test]
fn test_parse_is_http_variations() {
    let test_cases = vec![
        ("is-http:", Some(true)),
        ("is-http: true", Some(true)),
        ("is-http: false", Some(true)), // Note: any value sets it to true
        ("is-http: anything", Some(true)),
    ];

    for (line, expected) in test_cases {
        let result = GreqHeader::parse(&vec![line]);
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.is_http, expected, "Failed for line: {}", line);
    }
}

#[test]
fn test_parse_duplicate_headers() {
    let lines = vec![
        "project: FirstProject",
        "project: SecondProject",
        "output-folder: /tmp1",
        "output-folder: /tmp2",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    // Should use the last value
    assert_eq!(header.project, "SecondProject");
    assert_eq!(header.output_folder, "/tmp2");
}

#[test]
fn test_parse_special_characters() {
    let lines = vec![
        "project: My-Project_123!",
        "output-folder: /path/with spaces/and-symbols@#$",
        "depends-on: request:with:colons",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, "My-Project_123!");
    assert_eq!(header.output_folder, "/path/with spaces/and-symbols@#$");
    assert_eq!(header.depends_on, Some("request:with:colons".to_string()));
}

#[test]
fn test_parse_unicode_characters() {
    let lines = vec![
        "project: プロジェクト名",
        "output-folder: /路径/测试",
        "depends-on: αβγ_request",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    assert_eq!(header.project, "プロジェクト名");
    assert_eq!(header.output_folder, "/路径/测试");
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
fn test_parse_colon_at_beginning() {
    let lines = vec![
        ":value_without_header_name",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_err());

    match result.unwrap_err() {
        GreqHeaderError::UnknownHeader { header_name } => {
            assert_eq!(header_name, "");
        }
        _ => panic!("Expected UnknownHeader error"),
    }
}

#[test]
fn test_parse_very_long_values() {
    let long_value = "a".repeat(1000);
    let formatted_line = format!("project: {}", long_value);
    let lines = vec![formatted_line.as_str()];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header.project, long_value);
}

