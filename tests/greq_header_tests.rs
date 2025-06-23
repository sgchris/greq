use greq::greq_object::greq_header::{GreqHeader, GreqHeaderError};
use greq::greq_object::traits::enrich_with_trait::EnrichWith;

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

    assert_eq!(header.project, Some("MyProject".to_string()));
    assert_eq!(header.output_folder, Some("/tmp/responses".to_string()));
    assert_eq!(header.output_file_name, Some("test.response".to_string()));
    assert_eq!(header.depends_on, Some("auth_request".to_string()));
    assert_eq!(header.base_request, Some("base.greq".to_string()));
    assert_eq!(header.is_http, true);
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

    assert_eq!(header.project, Some("MyProject".to_string()));
    assert_eq!(header.output_folder, Some("/tmp/responses".to_string()));
    assert_eq!(header.output_file_name, Some("test.response".to_string()));
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

    assert_eq!(header.project, Some("MyProject".to_string()));
    assert_eq!(header.output_folder, Some("/tmp".to_string()));
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

    assert_eq!(header.project, Some("MyProject".to_string()));
    assert_eq!(header.output_folder, Some("/tmp".to_string()));
    assert_eq!(header.depends_on, Some("auth".to_string()));
    assert_eq!(header.base_request, Some("base.greq".to_string()));
    assert_eq!(header.is_http, true);
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

    assert_eq!(header.project, Some("http://example.com:8080/path".to_string()));
    assert_eq!(header.output_folder, Some("C:\\Users\\test:folder".to_string()));
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
        "output-folder: /tmp1",
        "output-folder: /tmp2",
    ];

    let result = GreqHeader::parse(&lines);
    assert!(result.is_ok());
    let header = result.unwrap();

    // Should use the last value
    assert_eq!(header.project, Some("SecondProject".to_string()));
    assert_eq!(header.output_folder, Some("/tmp2".to_string()));
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

    assert_eq!(header.project, Some("My-Project_123!".to_string()));
    assert_eq!(header.output_folder, Some("/path/with spaces/and-symbols@#$".to_string()));
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

    assert_eq!(header.project, Some("プロジェクト名".to_string()));
    assert_eq!(header.output_folder, Some("/路径/测试".to_string()));
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

#[test]
fn test_enrich_with_empty_self_filled_other() {
    let mut self_header = GreqHeader::default();
    let other_header = GreqHeader {
        delimiter: '|',
        project: Some("test_project".to_string()),
        output_folder: Some("/test/output".to_string()),
        output_file_name: Some("test.response".to_string()),
        is_http: true,
        base_request: Some("base.greq".to_string()),
        depends_on: Some("dependency.greq".to_string()),
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, Some("test_project".to_string()));
    assert_eq!(self_header.output_folder, Some("/test/output".to_string()));
    assert_eq!(self_header.output_file_name, Some("test.response".to_string()));
    assert_eq!(self_header.is_http, false); // default value
    assert_eq!(self_header.base_request, Some("base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dependency.greq".to_string()));
}

#[test]
fn test_enrich_with_filled_self_empty_other() {
    let mut self_header = GreqHeader {
        delimiter: '#',
        project: Some("self_project".to_string()),
        output_folder: Some("/self/output".to_string()),
        output_file_name: Some("self.response".to_string()),
        is_http: false,
        base_request: Some("self_base.greq".to_string()),
        depends_on: Some("self_dependency.greq".to_string()),
    };
    let other_header = GreqHeader::default();

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Self values should remain unchanged
    assert_eq!(self_header.project, Some("self_project".to_string()));
    assert_eq!(self_header.output_folder, Some("/self/output".to_string()));
    assert_eq!(self_header.output_file_name, Some("self.response".to_string()));
    assert_eq!(self_header.is_http, false);
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("self_dependency.greq".to_string()));
}

#[test]
fn test_enrich_with_both_filled_self_takes_precedence() {
    let mut self_header = GreqHeader {
        delimiter: '#',
        project: Some("self_project".to_string()),
        output_folder: Some("/self/output".to_string()),
        output_file_name: Some("self.response".to_string()),
        is_http: false,
        base_request: Some("self_base.greq".to_string()),
        depends_on: Some("self_dependency.greq".to_string()),
    };

    let other_header = GreqHeader {
        delimiter: '|',
        project: Some("other_project".to_string()),
        output_folder: Some("/other/output".to_string()),
        output_file_name: Some("other.response".to_string()),
        is_http: true,
        base_request: Some("other_base.greq".to_string()),
        depends_on: Some("other_dependency.greq".to_string()),
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Self values should remain unchanged (precedence)
    assert_eq!(self_header.project, Some("self_project".to_string()));
    assert_eq!(self_header.output_folder, Some("/self/output".to_string()));
    assert_eq!(self_header.output_file_name, Some("self.response".to_string()));
    assert_eq!(self_header.is_http, false);
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("self_dependency.greq".to_string()));
}

#[test]
fn test_enrich_with_partial_merge() {
    let mut self_header = GreqHeader {
        delimiter: '=',
        project: Some("self_project".to_string()),
        output_folder: None, // empty -> None
        output_file_name: Some("self.response".to_string()),
        is_http: false, // None -> false (default)
        base_request: Some("self_base.greq".to_string()),
        depends_on: None,
    };
    let other_header = GreqHeader {
        delimiter: '|',
        project: Some("other_project".to_string()),
        output_folder: Some("/other/output".to_string()),
        output_file_name: Some("other.response".to_string()),
        is_http: true,
        base_request: Some("other_base.greq".to_string()),
        depends_on: Some("other_dependency.greq".to_string()),
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, Some("self_project".to_string())); // unchanged (has value)
    assert_eq!(self_header.output_folder, Some("/other/output".to_string())); // merged (was empty)
    assert_eq!(self_header.output_file_name, Some("self.response".to_string())); // unchanged (has value)
    assert_eq!(self_header.is_http, false); // merged (was None)
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string())); // unchanged (has value)
    assert_eq!(self_header.depends_on, Some("other_dependency.greq".to_string())); // merged (was None)
}

#[test]
fn test_enrich_with_option_fields_none_to_some() {
    let mut self_header = GreqHeader {
        is_http: false,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };
    let other_header = GreqHeader {
        is_http: true,
        base_request: Some("base.greq".to_string()),
        depends_on: Some("dep.greq".to_string()),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.is_http, false); // remains unchanged
    assert_eq!(self_header.base_request, Some("base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dep.greq".to_string()));
}

#[test]
fn test_enrich_with_option_fields_some_to_none() {
    let mut self_header = GreqHeader {
        is_http: false,
        base_request: Some("self_base.greq".to_string()),
        depends_on: Some("self_dep.greq".to_string()),
        ..Default::default()
    };
    let other_header = GreqHeader {
        is_http: true,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Self values should remain unchanged
    assert_eq!(self_header.is_http, false); // remains unchanged
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("self_dep.greq".to_string()));
}

#[test]
fn test_enrich_with_both_none_remains_none() {
    let mut self_header = GreqHeader {
        is_http: false,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };
    let other_header = GreqHeader {
        is_http: false,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.is_http, false);
    assert_eq!(self_header.base_request, None);
    assert_eq!(self_header.depends_on, None);
}

#[test]
fn test_enrich_with_identical_objects() {
    let mut self_header = GreqHeader {
        delimiter: '=',
        project: Some("project".to_string()),
        output_folder: Some("/output".to_string()),
        output_file_name: Some("test.response".to_string()),
        is_http: true,
        base_request: Some("base.greq".to_string()),
        depends_on: Some("dep.greq".to_string()),
    };

    let other_header = self_header.clone();

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Should remain identical
    assert_eq!(self_header.project, Some("project".to_string()));
    assert_eq!(self_header.output_folder, Some("/output".to_string()));
    assert_eq!(self_header.output_file_name, Some("test.response".to_string()));
    assert_eq!(self_header.is_http, true);
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
        project: None,
        output_folder: Some("   ".to_string()), // whitespace only
        ..Default::default()
    };
    let other_header = GreqHeader {
        project: Some("other_project".to_string()),
        output_folder: Some("/other/output".to_string()),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, Some("other_project".to_string())); // merged (was empty)
    // Note: whitespace-only strings are not considered empty by is_empty()
    assert_eq!(self_header.output_folder, Some("   ".to_string())); // not merged (has whitespace)
}

#[test]
fn test_enrich_with_special_characters() {
    let mut self_header = GreqHeader::default();
    let other_header = GreqHeader {
        project: Some("project-with-dashes".to_string()),
        output_folder: Some("/path/with spaces/and-dashes".to_string()),
        output_file_name: Some("file_name.with.dots".to_string()),
        base_request: Some("base-request.greq".to_string()),
        depends_on: Some("dependency_file.greq".to_string()),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, Some("project-with-dashes".to_string()));
    assert_eq!(self_header.output_folder, Some("/path/with spaces/and-dashes".to_string()));
    assert_eq!(self_header.output_file_name, Some("file_name.with.dots".to_string()));
    assert_eq!(self_header.base_request, Some("base-request.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dependency_file.greq".to_string()));
}

#[test]
fn test_enrich_with_preserves_original_string_and_delimiter() {
    let mut self_header = GreqHeader {
        delimiter: '#',
        ..Default::default()
    };
    let other_header = GreqHeader {
        delimiter: '|',
        project: Some("other_project".to_string()),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // original_string and delimiter should not be affected by merge
    assert_eq!(self_header.delimiter, '#');
    // but other fields should merge
    assert_eq!(self_header.project, Some("other_project".to_string()));
}

