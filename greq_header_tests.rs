
use std::{
    fs::File,
    env,
};
use greq::greq_object::{
    greq_header::{GreqHeader, GreqHeaderError},
    traits::enrich_with_trait::EnrichWith,
};

#[test]
fn test_parse_lines_success_scenarios() {
    // Test 1: Empty header lines (should return default)
    let empty_lines = vec![];
    let result = GreqHeader::parse_lines_into_greq_header_object(&greq::greq_object::greq_parser::strs_to_cows(&empty_lines));
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header, GreqHeader::default());

    // Test 2: Complete valid header with all properties
    let lines = vec![
        "project: MyProject",
        "depends-on: auth_request", 
        "extends: base.greq",
        "is-http: true",
        "delimiter: |"
    ];
    let result = GreqHeader::parse_lines_into_greq_header_object(
        &greq::greq_object::greq_parser::strs_to_cows(&lines)
    );
    assert!(result.is_ok());
    let header = result.unwrap();
    
    assert_eq!(header.project, Some("MyProject".to_string()));
    assert_eq!(header.depends_on, Some("auth_request".to_string()));
    assert_eq!(header.extends, Some("base.greq".to_string()));
    assert_eq!(header.is_http, true);
    assert_eq!(header.delimiter, '|');

    // Test 3: Case insensitive headers and whitespace handling
    let lines = vec![
        "  PROJECT  :  MyProject  ",
        "IS-HTTP: false",
        "   ",  // empty line
        "\t\t",  // whitespace only
        "extends:base.greq"  // no spaces around colon
    ];
    let result = GreqHeader::parse_lines_into_greq_header_object(&greq::greq_object::greq_parser::strs_to_cows(&lines));
    assert!(result.is_ok());
    let header = result.unwrap();
    
    assert_eq!(header.project, Some("MyProject".to_string()));
    assert_eq!(header.is_http, false);
    assert_eq!(header.extends, Some("base.greq".to_string()));

    // Test 4: Boolean variations and special characters
    let test_cases = vec![
        ("is-http: yes", true),
        ("is-http: no", false),
        ("is-http: 1", true),
        ("is-http: 0", false),
    ];
    
    for (line, expected) in test_cases {
        let result = GreqHeader::parse_lines_into_greq_header_object(&greq::greq_object::greq_parser::strs_to_cows(&vec![line]));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().is_http, expected);
    }

    // Test 5: Multiple colons and Unicode support
    let lines = vec![
        "project: http://example.com:8080/path",  // multiple colons
        "depends-on: プロジェクト名",  // Unicode
    ];
    let result = GreqHeader::parse_lines_into_greq_header_object(&greq::greq_object::greq_parser::strs_to_cows(&lines));
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header.project, Some("http://example.com:8080/path".to_string()));
    assert_eq!(header.depends_on, Some("プロジェクト名".to_string()));
}

#[test]
fn test_parse_lines_all_error_scenarios() {
    // Test 1: Missing colon
    let lines = vec!["invalid_line_without_colon"];
    let result = GreqHeader::parse_lines_into_greq_header_object(&greq::greq_object::greq_parser::strs_to_cows(&lines));
    assert!(matches!(result, Err(GreqHeaderError::LineHasNoColonSign { .. })));

    // Test 2: Empty header name (colon at beginning)
    let lines = vec![":value_without_header_name"];
    let result = GreqHeader::parse_lines_into_greq_header_object(&greq::greq_object::greq_parser::strs_to_cows(&lines));
    assert!(matches!(result, Err(GreqHeaderError::HeaderHasNoName { .. })));

    // Test 3: Empty header values
    let lines = vec!["project:"];
    let result = GreqHeader::parse_lines_into_greq_header_object(&greq::greq_object::greq_parser::strs_to_cows(&lines));
    assert!(matches!(result, Err(GreqHeaderError::HeaderHasNoValue { .. })));

    // Test 4: Unknown header
    let lines = vec!["unknown-header: some_value"];
    let result = GreqHeader::parse_lines_into_greq_header_object(&greq::greq_object::greq_parser::strs_to_cows(&lines));
    assert!(matches!(result, Err(GreqHeaderError::UnknownHeader { .. })));

    // Test 5: Invalid delimiter (more than one character)
    let lines = vec!["delimiter: ||"];
    let result = GreqHeader::parse_lines_into_greq_header_object(&greq::greq_object::greq_parser::strs_to_cows(&lines));
    assert!(matches!(result, Err(GreqHeaderError::InvalidHeaderValue { .. })));

    // Test 6: Invalid boolean values
    let invalid_bool_lines = vec![
        "is-http: maybe",
        "is-http: 2", 
        "is-http: unknown"
    ];
    
    for line in invalid_bool_lines {
        let result = GreqHeader::parse_lines_into_greq_header_object(&greq::greq_object::greq_parser::strs_to_cows(&vec![line]));
        assert!(matches!(result, Err(GreqHeaderError::InvalidHeaderValue { .. })));
    }
}

#[test]
fn test_full_parse_with_file_operations() {
    // Create temporary files for testing
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push("greq_test");
    std::fs::create_dir_all(&tmp_dir).unwrap();

    let mut base_file = tmp_dir.clone();
    base_file.push("base.greq");
    File::create(&base_file).unwrap();

    let mut depends_file = tmp_dir.clone();
    depends_file.push("depends.greq");
    File::create(&depends_file).unwrap();

    let mut current_file = tmp_dir.clone();
    current_file.push("current.greq");

    // Test 1: Successful parse with file validation
    let lines = vec![
        "project: TestProject",
        "extends: base",  // without .greq extension
        "depends-on: depends.greq"
    ];
    
    let result = GreqHeader::parse(&lines, current_file.to_str().unwrap(), None, None);
    assert!(result.is_ok());
    let header = result.unwrap();
    
    assert_eq!(header.project, Some("TestProject".to_string()));
    assert!(header.extends.is_some());
    assert!(header.depends_on.is_some());

    // Test 2: File not found error for extends
    let lines = vec!["extends: nonexistent"];
    let result = GreqHeader::parse(&lines, current_file.to_str().unwrap(), None, None);
    assert!(matches!(result, Err(GreqHeaderError::FileDoesNotExist { .. })));

    // Test 3: File not found error for depends-on
    let lines = vec!["depends-on: nonexistent.greq"];
    let result = GreqHeader::parse(&lines, current_file.to_str().unwrap(), None, None);
    assert!(matches!(result, Err(GreqHeaderError::FileDoesNotExist { .. })));

    // Test 4: Duplicate headers (should use last value)
    let lines = vec![
        "project: FirstProject",
        "project: SecondProject"
    ];
    let result = GreqHeader::parse(&lines, current_file.to_str().unwrap(), None, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().project, Some("SecondProject".to_string()));

    // Clean up
    std::fs::remove_file(&base_file).unwrap();
    std::fs::remove_file(&depends_file).unwrap();
    std::fs::remove_dir(&tmp_dir).unwrap();
}

#[test]
fn test_enrich_with_comprehensive() {
    // Test 1: Enrich empty header with complete header
    let mut empty_header = GreqHeader::default();
    let complete_header = GreqHeader {
        absolute_path: "/path/to/file.greq".to_string(),
        delimiter: '|',
        project: Some("BaseProject".to_string()),
        is_http: true,
        extends: Some("base.greq".to_string()),
        depends_on: Some("dependency.greq".to_string()),
    };

    let result = empty_header.enrich_with(&complete_header);
    assert!(result.is_ok());
    
    // Only project and depends_on should be enriched (as per implementation)
    assert_eq!(empty_header.project, Some("BaseProject".to_string()));
    assert_eq!(empty_header.depends_on, Some("dependency.greq".to_string()));
    // Other fields should remain default
    assert_eq!(empty_header.delimiter, '='); // default
    assert_eq!(empty_header.is_http, false); // default
    assert_eq!(empty_header.extends, None); // default

    // Test 2: Enrich existing header (should not override existing values)
    let mut existing_header = GreqHeader {
        absolute_path: "/existing/path.greq".to_string(),
        delimiter: '*',
        project: Some("ExistingProject".to_string()),
        is_http: true,
        extends: Some("existing_base.greq".to_string()),
        depends_on: Some("existing_dep.greq".to_string()),
    };

    let merge_header = GreqHeader {
        project: Some("NewProject".to_string()),
        depends_on: Some("NewDependency".to_string()),
        ..Default::default()
    };

    let result = existing_header.enrich_with(&merge_header);
    assert!(result.is_ok());
    
    // Existing values should be preserved (not overridden)
    assert_eq!(existing_header.project, Some("ExistingProject".to_string()));
    assert_eq!(existing_header.depends_on, Some("existing_dep.greq".to_string()));

    // Test 3: Partial enrichment (only missing fields get enriched)
    let mut partial_header = GreqHeader {
        project: Some("PartialProject".to_string()),
        depends_on: None,
        ..Default::default()
    };

    let enrich_header = GreqHeader {
        project: Some("ShouldNotOverride".to_string()),
        depends_on: Some("ShouldBeAdded".to_string()),
        ..Default::default()
    };

    let result = partial_header.enrich_with(&enrich_header);
    assert!(result.is_ok());
    
    // Existing project should be preserved, missing depends_on should be added
    assert_eq!(partial_header.project, Some("PartialProject".to_string()));
    assert_eq!(partial_header.depends_on, Some("ShouldBeAdded".to_string()));

    // Test 4: Enrich with None values (should not change anything)
    let mut header_with_values = GreqHeader {
        project: Some("KeepThis".to_string()),
        depends_on: Some("KeepThisToo".to_string()),
        ..Default::default()
    };

    let empty_enrich_header = GreqHeader::default(); // all None values

    let result = header_with_values.enrich_with(&empty_enrich_header);
    assert!(result.is_ok());
    
    // Values should remain unchanged
    assert_eq!(header_with_values.project, Some("KeepThis".to_string()));
    assert_eq!(header_with_values.depends_on, Some("KeepThisToo".to_string()));
}
