use greq::greq_object::greq::Greq;
use greq::constants::{ DEFAULT_DELIMITER_CHAR };

#[test]
fn test_parse_section_splitting_with_default_delimiter() {
    let content = r#"project: TestProject
====
GET /test HTTP/1.1
Host: example.com
====
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.sections_delimiter, DEFAULT_DELIMITER_CHAR);
}

#[test]
fn test_parse_section_splitting_with_custom_delimiter() {
    let content = r#"delimiter: +
project: TestProject
++++
GET /test HTTP/1.1
Host: example.com
++++
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.sections_delimiter, '+');
}

#[test]
fn test_parse_delimiter_in_first_line() {
    let content = r#"delimiter: #
####
GET /test HTTP/1.1
Host: example.com
####
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.sections_delimiter, '#');
}

#[test]
fn test_parse_delimiter_mixed_with_other_headers() {
    let content = r#"project: TestProject
delimiter: @
@@@@
GET /test HTTP/1.1
Host: example.com
@@@@
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.sections_delimiter, '@');
    assert_eq!(result.header.project, Some("TestProject".to_string()));
}

// Section count edge cases

#[test]
fn test_parse_no_sections() {
    let content = "project: TestProject";

    let result = Greq::parse(content);
    assert!(result.is_err()); // Should fail with insufficient sections
}

#[test]
fn test_parse_one_section_only() {
    let content = r#"project: TestProject
====
GET /test HTTP/1.1
Host: example.com"#;

    let result = Greq::parse(content);
    assert!(result.is_err()); // Should fail - missing footer section
}

#[test]
fn test_parse_four_sections() {
    let content = r#"project: TestProject
====
GET /test HTTP/1.1
Host: example.com
====
status-code equals 200
====
extra section"#;

    let result = Greq::parse(content);
    assert!(result.is_err()); // Should fail - too many sections
}

#[test]
fn test_parse_five_sections() {
    let content = r#"project: TestProject
====
GET /test HTTP/1.1
Host: example.com
====
status-code equals 200
====
fourth section
====
fifth section"#;

    let result = Greq::parse(content);
    assert!(result.is_err()); // Should fail - way too many sections
}

// Delimiter edge cases

#[test]
fn test_parse_delimiter_not_repeated_four_times() {
    let content = r#"project: TestProject
===
GET /test HTTP/1.1
Host: example.com
===
status-code equals: 200"#;

    let result = Greq::parse(content);
    assert!(result.is_err()); // Should fail - delimiter must be at least 4 chars
}

#[test]
fn test_parse_delimiter_exactly_four_times() {
    let content = r#"project: TestProject
====
GET /test HTTP/1.1
Host: example.com
====
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.content.method, "GET");
}

#[test]
fn test_parse_delimiter_more_than_four_times() {
    let content = r#"delimiter: #
project: TestProject
########
GET /test HTTP/1.1
Host: example.com
########
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.sections_delimiter, '#');
}

#[test]
fn test_parse_mixed_delimiter_lengths() {
    let content = r#"delimiter: +
project: TestProject
++++
GET /test HTTP/1.1
Host: example.com
++++++++
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.content.method, "GET");
}

#[test]
fn test_parse_delimiter_with_spaces_around() {
    let content = r#"project: TestProject
  ====  
GET /test HTTP/1.1
Host: example.com
  ====  
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.content.method, "GET");
}

#[test]
fn test_parse_delimiter_mixed_with_content() {
    let content = r#"project: TestProject
====
GET /test HTTP/1.1
Host: example.com
Authorization: Bearer ====token====
====
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert!(result.content.headers.get("Authorization").unwrap().contains("===="));
}

// Whitespace and newline edge cases

#[test]
fn test_parse_empty_lines_between_sections() {
    let content = r#"project: TestProject

====

GET /test HTTP/1.1
Host: example.com

====

status-code equals: 200

"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.content.method, "GET");
}

#[test]
fn test_parse_multiple_empty_lines() {
    let content = r#"project: TestProject



====



GET /test HTTP/1.1
Host: example.com



====



status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.content.method, "GET");
}

#[test]
fn test_parse_only_whitespace_sections() {
    let content = r#"   
====
GET /test HTTP/1.1
Host: example.com
====
   "#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.footer.conditions.len(), 0); // Empty footer
}

#[test]
fn test_parse_tabs_and_spaces_mixed() {
    let content = "project: TestProject\t\n\t\t====\t\t\nGET /test HTTP/1.1\nHost: example.com\n\t====\t\nstatus-code equals: 200";

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.content.method, "GET");
}

// Content boundaries and special cases

#[test]
fn test_parse_content_with_delimiter_sequence() {
    let content = r#"project: TestProject
====
POST /test HTTP/1.1
Host: example.com
Content-Type: text/plain

This content contains ====
multiple ==== sequences
that should not break parsing
====
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert!(result.content.body.contains("===="));
    assert!(result.content.body.contains("multiple"));
}

#[test]
fn test_parse_footer_with_delimiter_in_values() {
    let content = r#"project: TestProject
====
GET /test HTTP/1.1
Host: example.com
====
response-body contains: "message====value"
headers.custom-header equals: "data====more-data""#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.footer.conditions.len(), 2);
    assert!(result.footer.conditions[0].value.contains("===="));
}

// File content preservation

#[test]
fn test_parse_preserves_original_content() {
    let content = r#"project: TestProject
delimiter: @
@@@@
GET /test HTTP/1.1
Host: example.com
@@@@
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.file_contents, content);
}

#[test]
fn test_parse_preserves_content_with_special_chars() {
    let content = "project: Test\tProject\n====\nGET /test HTTP/1.1\nHost: example.com\n====\nstatus-code equals: 200";

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.file_contents, content);
}

// Extreme edge cases

#[test]
fn test_parse_single_line_sections() {
    let content = "project: Test\n====\nGET / HTTP/1.1\nHost: example.com\n====\nstatus-code equals: 200";

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.content.uri, "/");
}

#[test]
fn test_parse_empty_string() {
    let result = Greq::parse("");
    assert!(result.is_err());
}

#[test]
fn test_parse_only_delimiters() {
    let content = "====\n====\n====";

    let result = Greq::parse(content);
    assert!(result.is_err()); // Should fail due to content parsing
}

#[test]
fn test_parse_only_newlines() {
    let content = "\n\n\n\n";

    let result = Greq::parse(content);
    assert!(result.is_err());
}

#[test]
fn test_parse_very_long_sections() {
    let long_header = format!("project: {}", "A".repeat(10000));
    let long_body = "B".repeat(50000);
    let content = format!(r#"{}
====
POST /upload HTTP/1.1
Host: example.com

{}
====
status-code equals: 201"#, long_header, long_body);

    let result = Greq::parse(&content).unwrap();
    assert!(result.header.project.as_ref().unwrap().len() > 9000);
    assert!(result.content.body.len() > 40000);
}

// Boundary conditions for section detection

#[test]
fn test_parse_delimiter_at_start_of_file() {
    let content = r#"====
GET /test HTTP/1.1
Host: example.com
====
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.content.method, "GET");
}

#[test]
fn test_parse_delimiter_at_end_of_file() {
    let content = r#"project: TestProject
====
GET /test HTTP/1.1
Host: example.com
====
status-code equals: 200
===="#;

    let result = Greq::parse(content);
    assert!(result.is_err()); // Should fail - too many sections
}

#[test]
fn test_parse_no_newline_at_end() {
    let content = r#"project: TestProject
====
GET /test HTTP/1.1
Host: example.com
====
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.footer.conditions.len(), 1);
}

#[test]
fn test_parse_different_line_endings() {
    // Test with \r\n (Windows line endings)
    let content = "project: TestProject\r\n====\r\nGET /test HTTP/1.1\r\nHost: example.com\r\n====\r\nstatus-code equals: 200";

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.content.method, "GET");
}

// Nested delimiter patterns

#[test]
fn test_parse_delimiter_subset_in_content() {
    let content = r#"project: TestProject
====
POST /test HTTP/1.1
Host: example.com

= single equal
== double equal  
=== triple equal
===== five equals
====
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert!(result.content.body.contains("=== triple equal"));
    assert!(result.content.body.contains("===== five equals"));
}

#[test]
fn test_parse_custom_delimiter_subset_patterns() {
    let content = r#"delimiter: +
project: TestProject
++++
POST /test HTTP/1.1
Host: example.com

+ single plus
++ double plus
+++ triple plus
+++++ five plus
++++
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert!(result.content.body.contains("+++ triple plus"));
    assert!(result.content.body.contains("+++++ five plus"));
}

// Unicode and special character handling

#[test]
fn test_parse_unicode_content() {
    let content = r#"project: тест
====
GET /тест HTTP/1.1
Host: example.com

{"message": "🚀 успех! 测试"}
====
response-body contains: 🚀"#;

    let result = Greq::parse(content).unwrap();
    assert!(result.header.project.as_ref().unwrap().contains("тест"));
    assert!(result.content.body.contains("🚀"));
}

#[test]
#[ignore] // Unicode delimiter will be supported in future versions
fn test_parse_unicode_delimiter() {
    let content = r#"delimiter: ♦
project: TestProject
♦♦♦♦
GET /test HTTP/1.1
Host: example.com
♦♦♦♦
status-code equals: 200"#;

    let result = Greq::parse(content).unwrap();
    assert_eq!(result.sections_delimiter, '♦');
}
