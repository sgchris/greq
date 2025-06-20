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
        "output-folder:",
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
        "output-folder: /tmp/responses",
        "output-file-name: test.response",
        "depends-on: auth_request",
        "base-request: base.greq",
        "is-http: true",
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
        "IS-HTTP:true",
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
        ("is-http: false", Some(false)),
        ("is-http: true", Some(true)),
        ("is-http: yes", Some(true)),
        ("is-http: no", Some(false)),
        ("is-http: 1", Some(true)),
        ("is-http: 0", Some(false)),
        ("is-http: false", Some(false)),
        ("is-http: true", Some(true)),
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
fn test_parse_very_long_values() {
    let long_value = "a".repeat(1000);
    let formatted_line = format!("project: {}", long_value);
    let lines = vec![formatted_line.as_str()];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header.project, long_value);
}

#[test]
fn test_enrich_with_empty_self_filled_other() {
    let mut self_header = GreqHeader::default();
    let other_header = GreqHeader {
        original_string: "test".to_string(),
        delimiter: '|',
        project: "test_project".to_string(),
        output_folder: "/test/output".to_string(),
        output_file_name: "test.response".to_string(),
        is_http: Some(true),
        base_request: Some("base.greq".to_string()),
        depends_on: Some("dependency.greq".to_string()),
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, "test_project");
    assert_eq!(self_header.output_folder, "/test/output");
    assert_eq!(self_header.output_file_name, "test.response");
    assert_eq!(self_header.is_http, Some(true));
    assert_eq!(self_header.base_request, Some("base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dependency.greq".to_string()));
}

#[test]
fn test_enrich_with_filled_self_empty_other() {
    let mut self_header = GreqHeader {
        original_string: "self".to_string(),
        delimiter: '#',
        project: "self_project".to_string(),
        output_folder: "/self/output".to_string(),
        output_file_name: "self.response".to_string(),
        is_http: Some(false),
        base_request: Some("self_base.greq".to_string()),
        depends_on: Some("self_dependency.greq".to_string()),
    };
    let other_header = GreqHeader::default();

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Self values should remain unchanged
    assert_eq!(self_header.project, "self_project");
    assert_eq!(self_header.output_folder, "/self/output");
    assert_eq!(self_header.output_file_name, "self.response");
    assert_eq!(self_header.is_http, Some(false));
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("self_dependency.greq".to_string()));
}

#[test]
fn test_enrich_with_both_filled_self_takes_precedence() {
    let mut self_header = GreqHeader {
        original_string: "self".to_string(),
        delimiter: '#',
        project: "self_project".to_string(),
        output_folder: "/self/output".to_string(),
        output_file_name: "self.response".to_string(),
        is_http: Some(false),
        base_request: Some("self_base.greq".to_string()),
        depends_on: Some("self_dependency.greq".to_string()),
    };
    let other_header = GreqHeader {
        original_string: "other".to_string(),
        delimiter: '|',
        project: "other_project".to_string(),
        output_folder: "/other/output".to_string(),
        output_file_name: "other.response".to_string(),
        is_http: Some(true),
        base_request: Some("other_base.greq".to_string()),
        depends_on: Some("other_dependency.greq".to_string()),
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Self values should remain unchanged (precedence)
    assert_eq!(self_header.project, "self_project");
    assert_eq!(self_header.output_folder, "/self/output");
    assert_eq!(self_header.output_file_name, "self.response");
    assert_eq!(self_header.is_http, Some(false));
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("self_dependency.greq".to_string()));
}

#[test]
fn test_enrich_with_partial_merge() {
    let mut self_header = GreqHeader {
        original_string: "".to_string(),
        delimiter: '=',
        project: "self_project".to_string(), // has value
        output_folder: "".to_string(), // empty
        output_file_name: "self.response".to_string(), // has value
        is_http: None, // None
        base_request: Some("self_base.greq".to_string()), // has value
        depends_on: None, // None
    };
    let other_header = GreqHeader {
        original_string: "other".to_string(),
        delimiter: '|',
        project: "other_project".to_string(),
        output_folder: "/other/output".to_string(),
        output_file_name: "other.response".to_string(),
        is_http: Some(true),
        base_request: Some("other_base.greq".to_string()),
        depends_on: Some("other_dependency.greq".to_string()),
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, "self_project"); // unchanged (has value)
    assert_eq!(self_header.output_folder, "/other/output"); // merged (was empty)
    assert_eq!(self_header.output_file_name, "self.response"); // unchanged (has value)
    assert_eq!(self_header.is_http, Some(true)); // merged (was None)
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string())); // unchanged (has value)
    assert_eq!(self_header.depends_on, Some("other_dependency.greq".to_string())); // merged (was None)
}

#[test]
fn test_enrich_with_option_fields_none_to_some() {
    let mut self_header = GreqHeader {
        is_http: None,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };
    let other_header = GreqHeader {
        is_http: Some(true),
        base_request: Some("base.greq".to_string()),
        depends_on: Some("dep.greq".to_string()),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.is_http, Some(true));
    assert_eq!(self_header.base_request, Some("base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dep.greq".to_string()));
}

#[test]
fn test_enrich_with_option_fields_some_to_none() {
    let mut self_header = GreqHeader {
        is_http: Some(false),
        base_request: Some("self_base.greq".to_string()),
        depends_on: Some("self_dep.greq".to_string()),
        ..Default::default()
    };
    let other_header = GreqHeader {
        is_http: None,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Self values should remain unchanged
    assert_eq!(self_header.is_http, Some(false));
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("self_dep.greq".to_string()));
}

#[test]
fn test_enrich_with_both_none_remains_none() {
    let mut self_header = GreqHeader {
        is_http: None,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };
    let other_header = GreqHeader {
        is_http: None,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.is_http, None);
    assert_eq!(self_header.base_request, None);
    assert_eq!(self_header.depends_on, None);
}

#[test]
fn test_enrich_with_identical_objects() {
    let mut self_header = GreqHeader {
        original_string: "test".to_string(),
        delimiter: '=',
        project: "project".to_string(),
        output_folder: "/output".to_string(),
        output_file_name: "test.response".to_string(),
        is_http: Some(true),
        base_request: Some("base.greq".to_string()),
        depends_on: Some("dep.greq".to_string()),
    };
    let other_header = GreqHeader {
        original_string: "test".to_string(),
        delimiter: '=',
        project: "project".to_string(),
        output_folder: "/output".to_string(),
        output_file_name: "test.response".to_string(),
        is_http: Some(true),
        base_request: Some("base.greq".to_string()),
        depends_on: Some("dep.greq".to_string()),
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Should remain identical
    assert_eq!(self_header.project, "project");
    assert_eq!(self_header.output_folder, "/output");
    assert_eq!(self_header.output_file_name, "test.response");
    assert_eq!(self_header.is_http, Some(true));
    assert_eq!(self_header.base_request, Some("base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dep.greq".to_string()));
}

#[test]
fn test_enrich_with_both_empty() {
    let mut self_header = GreqHeader::default();
    let other_header = GreqHeader::default();

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Should remain default
    assert_eq!(self_header, GreqHeader::default());
}

#[test]
fn test_enrich_with_whitespace_handling() {
    let mut self_header = GreqHeader {
        project: "".to_string(),
        output_folder: "   ".to_string(), // whitespace only
        ..Default::default()
    };
    let other_header = GreqHeader {
        project: "other_project".to_string(),
        output_folder: "/other/output".to_string(),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, "other_project"); // merged (was empty)
    // Note: whitespace-only strings are not considered empty by is_empty()
    assert_eq!(self_header.output_folder, "   "); // not merged (has whitespace)
}

#[test]
fn test_enrich_with_special_characters() {
    let mut self_header = GreqHeader::default();
    let other_header = GreqHeader {
        project: "project-with-dashes".to_string(),
        output_folder: "/path/with spaces/and-dashes".to_string(),
        output_file_name: "file_name.with.dots".to_string(),
        base_request: Some("base-request.greq".to_string()),
        depends_on: Some("dependency_file.greq".to_string()),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, "project-with-dashes");
    assert_eq!(self_header.output_folder, "/path/with spaces/and-dashes");
    assert_eq!(self_header.output_file_name, "file_name.with.dots");
    assert_eq!(self_header.base_request, Some("base-request.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dependency_file.greq".to_string()));
}

#[test]
fn test_enrich_with_preserves_original_string_and_delimiter() {
    let mut self_header = GreqHeader {
        original_string: "self_original".to_string(),
        delimiter: '#',
        ..Default::default()
    };
    let other_header = GreqHeader {
        original_string: "other_original".to_string(),
        delimiter: '|',
        project: "other_project".to_string(),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // original_string and delimiter should not be affected by merge
    assert_eq!(self_header.original_string, "self_original");
    assert_eq!(self_header.delimiter, '#');
    // but other fields should merge
    assert_eq!(self_header.project, "other_project");
}

