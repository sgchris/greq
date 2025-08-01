use std::fs;
use std::borrow::Cow;
use std::path::Path;
use greq::greq_object::greq_parser::{
    get_header_section_lines, is_line_only_from_char, strs_to_cows, 
    replace_placeholders_in_lines, resolve_and_check_file_exists
};
use greq::greq_object::greq_response::GreqResponse;

#[test]
fn test_parse_header_section_basic() {
    // Test basic header parsing until delimiter
    let content = "GET /api/test\nHost: example.com\nContent-Type: application/json\n####\nbody content";
    let result = get_header_section_lines(content).unwrap();
    
    assert_eq!(result, vec!["GET /api/test", "Host: example.com", "Content-Type: application/json"]);
}

#[test]
fn test_parse_header_section_with_comments() {
    // Test header parsing with comment lines
    let content = "-- This is a comment\nGET /api/test\n-- Another comment\nHost: example.com\n####\nbody";
    let result = get_header_section_lines(content).unwrap();
    
    assert_eq!(result, vec!["GET /api/test", "Host: example.com"]);
}

#[test]
fn test_parse_header_section_empty_lines() {
    // Test header parsing with empty lines at start and middle
    let content = "\n\n  \nproject: greqtest\n\nextends: another.greq\n\n@@@@\nnext section starts here";
    let result = get_header_section_lines(content).unwrap();

    assert_eq!(result, vec!["project: greqtest", "extends: another.greq"]);
}

#[test]
fn test_parse_header_section_no_delimiter() {
    // Test header parsing when no delimiter is found
    let content = "GET /api/test\nHost: example.com\nContent-Type: application/json";
    let result = get_header_section_lines(content).unwrap();
    
    assert_eq!(result, vec!["GET /api/test", "Host: example.com", "Content-Type: application/json"]);
}

#[test]
fn test_parse_header_section_various_delimiters() {
    // Test different delimiter characters
    let test_cases = vec![
        ("line1\nline2\n====\nbody", vec!["line1", "line2"]),
        ("line1\nline2\n****\nbody", vec!["line1", "line2"]),
        ("line1\nline2\n@@@@\nbody", vec!["line1", "line2"]),
        ("line1\nline2\n!!!!\nbody", vec!["line1", "line2"]),
    ];
    
    for (content, expected) in test_cases {
        let result = get_header_section_lines(content).unwrap();
        assert_eq!(result, expected);
    }
}

#[test]
fn test_is_line_only_from_char_basic() {
    // Test basic functionality
    assert!(is_line_only_from_char("####", '#'));
    assert!(is_line_only_from_char("@@@@", '@'));
    assert!(is_line_only_from_char("****", '*'));
    assert!(is_line_only_from_char("====", '='));
}

#[test]
fn test_is_line_only_from_char_with_whitespace() {
    // Test with whitespace mixed in
    assert!(is_line_only_from_char("  ####  ", '#'));
    assert!(is_line_only_from_char("\t@@@@\t", '@'));
    assert!(is_line_only_from_char(" # # # ", '#'));
    assert!(is_line_only_from_char("   ", ' '));
}

#[test]
fn test_is_line_only_from_char_false_cases() {
    // Test cases that should return false
    assert!(!is_line_only_from_char("####text", '#'));
    assert!(!is_line_only_from_char("text####", '#'));
    assert!(!is_line_only_from_char("##text##", '#'));
    assert!(!is_line_only_from_char("abc", '#'));
    assert!(!is_line_only_from_char("", '#'));
}

#[test]
fn test_strs_to_cows_basic() {
    // Test basic conversion from Vec<&str> to Vec<Cow<str>>
    let strs = vec!["line1", "line2", "line3"];
    let cows = strs_to_cows(&strs);
    
    assert_eq!(cows.len(), 3);
    assert_eq!(cows[0], "line1");
    assert_eq!(cows[1], "line2");
    assert_eq!(cows[2], "line3");
    
    // Verify they are borrowed initially
    for cow in &cows {
        assert!(matches!(cow, Cow::Borrowed(_)));
    }
}

#[test]
fn test_strs_to_cows_empty() {
    // Test with empty vector
    let strs: Vec<&str> = vec![];
    let cows = strs_to_cows(&strs);
    
    assert!(cows.is_empty());
}

#[test]
fn test_replace_placeholders_basic() {
    // Test basic placeholder replacement
    let mut lines = vec![
        Cow::from("GET /api/$(dep.header.endpoint)"),
        Cow::from("Host: $(dep.header.host)"),
        Cow::from("Authorization: Bearer $(dependency.header.token)"),
    ];
    let mut response = GreqResponse::default();
    response.headers.insert("endpoint".to_string(), "users".to_string());
    response.headers.insert("host".to_string(), "api.example.com".to_string());
    response.headers.insert("token".to_string(), "abc123".to_string());
    
    replace_placeholders_in_lines(&mut lines, &response);
    
    assert_eq!(lines[0], "GET /api/users");
    assert_eq!(lines[1], "Host: api.example.com");
    assert_eq!(lines[2], "Authorization: Bearer abc123");
}

#[test]
fn test_replace_placeholders_no_matches() {
    // Test when no placeholders exist
    let mut lines = vec![
        Cow::from("GET /api/test"),
        Cow::from("Host: example.com"),
    ];
    
    let response = GreqResponse::default();
    replace_placeholders_in_lines(&mut lines, &response);
    
    // Lines should remain unchanged and borrowed
    assert_eq!(lines[0], "GET /api/test");
    assert_eq!(lines[1], "Host: example.com");
    assert!(matches!(lines[0], Cow::Borrowed(_)));
    assert!(matches!(lines[1], Cow::Borrowed(_)));
}

#[test]
fn test_replace_placeholders_missing_vars() {
    // Test when variables are not defined (should return empty string)
    let mut lines = vec![
        Cow::from("GET /api/$(missing_var)"),
        Cow::from("Token: $(undefined)"),
    ];
    
    let response = GreqResponse::default();
    println!("[DEBUG] lines: {:?}", lines);
    replace_placeholders_in_lines(&mut lines, &response);
    println!("[DEBUG] lines after: {:?}", lines);
    
    assert_eq!(lines[0], "GET /api/");
    assert_eq!(lines[1], "Token: ");
}

#[test]
fn test_replace_placeholders_escaped() {
    // Test escaped placeholders (should not be replaced)
    let mut lines = vec![
        Cow::from("Normal: $(dep.header.var)"),
        Cow::from("Escaped: \\$(dep.header.var1)"),
        Cow::from("Mixed: $(dep.header.var1) and \\$(dep.header.var2)"),
    ];
    
    let mut response = GreqResponse::default();
    response.headers.insert("var".to_string(), "value".to_string());
    response.headers.insert("var1".to_string(), "first".to_string());
    response.headers.insert("var2".to_string(), "second".to_string());
    
    replace_placeholders_in_lines(&mut lines, &response);
    
    assert_eq!(lines[0], "Normal: value");
    assert_eq!(lines[1], "Escaped: \\$(dep.header.var1)");
    assert_eq!(lines[2], "Mixed: first and \\$(dep.header.var2)");
}

#[test]
fn test_replace_placeholders_multiple_same_line() {
    // Test multiple placeholders in same line
    let mut lines = vec![
        Cow::from("$(dep.header.method) /$(dep.header.path)?user=$(dep.header.user)&token=$(dep.header.token)"),
    ];
    
    let mut response = GreqResponse::default();
    response.headers.insert("method".to_string(), "POST".to_string());
    response.headers.insert("path".to_string(), "api/data".to_string());
    response.headers.insert("user".to_string(), "john".to_string());
    response.headers.insert("token".to_string(), "xyz789".to_string());
    
    replace_placeholders_in_lines(&mut lines, &response);
    
    assert_eq!(lines[0], "POST /api/data?user=john&token=xyz789");
}

#[test]
fn test_resolve_file_absolute_path() {
    // Test with absolute path (create a temp file for testing)
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("greq_test_file.greq");
    fs::write(&test_file, "test content").unwrap();
    
    let absolute_path = test_file.to_str().unwrap();
    let result = resolve_and_check_file_exists(absolute_path, None).unwrap();
    
    assert!(Path::new(&result).is_absolute());
    assert!(Path::new(&result).exists());
    
    // Clean up
    fs::remove_file(&test_file).ok();
}

#[test]
fn test_resolve_file_relative_path_current_dir() {
    // Test with relative path using current directory
    // Create a temp file in current directory
    let test_file = "greq_test_relative.txt.greq";
    fs::write(test_file, "test content").unwrap();
    
    let result = resolve_and_check_file_exists(test_file, None).unwrap();
    
    assert!(Path::new(&result).is_absolute());
    assert!(Path::new(&result).exists());
    
    // Clean up
    fs::remove_file(test_file).ok();
}

#[test]
fn test_resolve_file_relative_path_with_base() {
    // Test with relative path using provided base path
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("greq_test_base.txt.greq");
    fs::write(&test_file, "test content").unwrap();
    
    let base_path = temp_dir.to_str().unwrap();
    let result = resolve_and_check_file_exists("greq_test_base.txt", Some(base_path)).unwrap();
    
    assert!(Path::new(&result).is_absolute());
    assert!(Path::new(&result).exists());
    
    // Clean up
    fs::remove_file(&test_file).ok();
}

#[test]
fn test_resolve_file_not_found() {
    // Test with non-existent file
    let result = resolve_and_check_file_exists("non_existent_file.txt", None);
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn test_resolve_file_invalid_base_path() {
    // Test with invalid base path
    let result = resolve_and_check_file_exists("test.txt", Some("/invalid/path/that/does/not/exist"));
    
    assert!(result.is_err());
}
