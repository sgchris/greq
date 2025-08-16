use crate::models::{GreqFile, Header, Content, Footer, RequestLine, Condition, ConditionKey, Operator};
use crate::error::{GreqError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use regex::Regex;

/// Parse a .greq file into a GreqFile structure
pub fn parse_greq_file<P: AsRef<Path>>(file_path: P) -> Result<GreqFile> {
    let file_path = file_path.as_ref();
    log::debug!("Parsing greq file: {file_path:?}");
    
    let content = fs::read_to_string(file_path)
        .map_err(|_| GreqError::FileNotFound(file_path.display().to_string()))?;
    
    let file_path_str = file_path.display().to_string();
    
    // Parse with line tracking
    parse_greq_content(&content, &file_path_str)
}

/// Parse greq content with file path for error reporting
fn parse_greq_content(content: &str, file_path: &str) -> Result<GreqFile> {
    let lines: Vec<&str> = content.lines().collect();
    let sections = split_into_sections(content, "=")?;
    
    if sections.len() < 2 {
        return Err(GreqError::Parse(format!("{}: Invalid file format: must have at least header and content sections", file_path)));
    }
    
    // Find section line numbers
    let section_starts = find_section_line_numbers(&lines, "=");
    
    let header = parse_header_with_lines(&sections[0], file_path, 1)?;
    let delimiter = header.delimiter.clone();
    
    // Re-split with custom delimiter if specified
    let (final_sections, final_section_starts) = if delimiter != "=" {
        let new_sections = split_into_sections(content, &delimiter)?;
        let new_starts = find_section_line_numbers(&lines, &delimiter);
        (new_sections, new_starts)
    } else {
        (sections, section_starts)
    };
    
    let content_start_line = if final_section_starts.is_empty() { 1 } else { final_section_starts[0] + 2 };
    let content_section = parse_content_with_lines(&final_sections[1], file_path, content_start_line)?;
    
    let footer = if final_sections.len() > 2 {
        let footer_start_line = if final_section_starts.len() >= 2 { final_section_starts[1] + 2 } else { content_start_line + final_sections[1].lines().count() + 2 };
        parse_footer_with_lines(&final_sections[2], file_path, footer_start_line)?
    } else {
        Footer::default()
    };
    
    Ok(GreqFile {
        header,
        content: content_section,
        footer,
        file_path: file_path.to_string(),
    })
}

/// Split file content into sections based on delimiter
fn split_into_sections(content: &str, delimiter: &str) -> Result<Vec<String>> {
    let delimiter_pattern = format!(r"(?m)^{}{{4,}}\s*$", regex::escape(delimiter));
    let regex = Regex::new(&delimiter_pattern)?;
    
    let sections: Vec<String> = regex
        .split(content)
        .map(|s| s.trim().to_string())
        .collect();
    
    log::debug!("Split content into {} sections using pattern '{}'", sections.len(), delimiter_pattern);
    for (i, section) in sections.iter().enumerate() {
        log::debug!("Section {i}: {section:?}");
    }
    
    if sections.len() < 2 {
        return Err(GreqError::Parse("File must contain at least 2 sections separated by delimiter".to_string()));
    }
    
    Ok(sections)
}

/// Find line numbers where section delimiters occur
fn find_section_line_numbers(lines: &[&str], delimiter: &str) -> Vec<usize> {
    let mut section_starts = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        if line.chars().all(|c| c == delimiter.chars().next().unwrap_or('=')) && line.len() >= 4 {
            section_starts.push(i + 1); // Convert to 1-based line numbers
        }
    }
    
    section_starts
}

/// Parse the header section with line number tracking
fn parse_header_with_lines(header_text: &str, file_path: &str, start_line: usize) -> Result<Header> {
    let mut header = Header::default();
    
    for (line_offset, line) in header_text.lines().enumerate() {
        let line_num = start_line + line_offset;
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 1..].trim();
            
            match key.as_str() {
                "project" => header.project = Some(value.to_string()),
                "is-http" => header.is_http = parse_bool(value)
                    .map_err(|_| GreqError::Parse(format!("{}:{}: Invalid boolean value '{}' for is-http", file_path, line_num, value)))?,
                "delimiter" => header.delimiter = value.to_string(),
                "extends" => header.extends = Some(value.to_string()),
                "number-of-retries" => header.number_of_retries = value.parse()
                    .map_err(|_| GreqError::Parse(format!("{}:{}: Invalid number '{}' for number-of-retries", file_path, line_num, value)))?,
                "depends-on" => header.depends_on = Some(value.to_string()),
                "allow-dependency-failure" => header.allow_dependency_failure = parse_bool(value)
                    .map_err(|_| GreqError::Parse(format!("{}:{}: Invalid boolean value '{}' for allow-dependency-failure", file_path, line_num, value)))?,
                "show-warnings" => header.show_warnings = parse_bool(value)
                    .map_err(|_| GreqError::Parse(format!("{}:{}: Invalid boolean value '{}' for show-warnings", file_path, line_num, value)))?,
                "timeout" => {
                    let timeout_ms: u64 = value.parse()
                        .map_err(|_| GreqError::Parse(format!("{}:{}: Invalid timeout value '{}'", file_path, line_num, value)))?;
                    header.timeout = Some(Duration::from_millis(timeout_ms));
                },
                _ => log::warn!("{}:{}: Unknown header property: {}", file_path, line_num, key),
            }
        } else {
            return Err(GreqError::Parse(format!("{}:{}: Missing colon in header line: '{}'", file_path, line_num, line)));
        }
    }
    
    // Validate header properties
    validate_header(&header)?;
    
    Ok(header)
}

/// Parse the content section with line number tracking
fn parse_content_with_lines(content_text: &str, file_path: &str, start_line: usize) -> Result<Content> {
    let lines: Vec<&str> = content_text.lines().collect();
    
    if lines.is_empty() {
        return Err(GreqError::Parse(format!("{}:{}: Content section cannot be empty", file_path, start_line)));
    }
    
    // Parse request line
    let request_line = parse_request_line_with_line(lines[0], file_path, start_line)?;
    
    // Parse headers and find body start
    let mut headers = HashMap::new();
    let mut body_start = lines.len();
    
    for (i, line) in lines.iter().enumerate().skip(1) {
        let line_num = start_line + i;
        let line = line.trim();
        
        if line.is_empty() {
            body_start = i + 1;
            break;
        }
        
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 1..].trim().to_string();
            headers.insert(key, value);
        } else {
            return Err(GreqError::Parse(format!("{}:{}: Missing colon in header line: '{}'", file_path, line_num, line)));
        }
    }
    
    // Parse body
    let body = if body_start < lines.len() {
        let body_lines: Vec<&str> = lines[body_start..].to_vec();
        if body_lines.iter().any(|line| !line.trim().is_empty()) {
            Some(body_lines.join("\n"))
        } else {
            None
        }
    } else {
        None
    };
    
    Ok(Content {
        request_line,
        headers,
        body,
    })
}

/// Parse request line with line number tracking
fn parse_request_line_with_line(line: &str, file_path: &str, line_num: usize) -> Result<RequestLine> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return Err(GreqError::Parse(format!("{}:{}: Invalid request line format, expected 'METHOD URI [VERSION]': '{}'", file_path, line_num, line)));
    }
    
    let method = parts[0].to_string();
    let uri = parts[1].to_string();
    let version = if parts.len() > 2 {
        parts[2].to_string()
    } else {
        "HTTP/1.1".to_string()
    };
    
    Ok(RequestLine { method, uri, version })
}

/// Parse the footer section with line number tracking
fn parse_footer_with_lines(footer_text: &str, file_path: &str, start_line: usize) -> Result<Footer> {
    let mut conditions = Vec::new();
    
    for (line_offset, line) in footer_text.lines().enumerate() {
        let line_num = start_line + line_offset;
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        
        match parse_condition_with_line(line, file_path, line_num) {
            Ok(condition) => conditions.push(condition),
            Err(e) => return Err(e),
        }
    }
    
    Ok(Footer { conditions })
}

/// Parse a single condition line with line number tracking
fn parse_condition_with_line(line: &str, file_path: &str, line_num: usize) -> Result<Condition> {
    let mut parts = line.split_whitespace().collect::<Vec<&str>>();
    let mut is_or = false;
    let mut is_not = false;
    let mut case_sensitive = false;
    
    // Check for prefixes
    if parts.first() == Some(&"or") {
        is_or = true;
        parts.remove(0);
    }
    
    if parts.first() == Some(&"not") {
        is_not = true;
        parts.remove(0);
    }
    
    if parts.len() < 3 {
        return Err(GreqError::Parse(format!("{}:{}: Invalid condition format, expected 'PROPERTY OPERATOR: VALUE': '{}'", file_path, line_num, line)));
    }
    
    // Find colon to separate operator from value (use first colon, not last)
    let colon_pos = line.find(':')
        .ok_or_else(|| GreqError::Parse(format!("{}:{}: Missing colon in condition: '{}'", file_path, line_num, line)))?;
    
    let before_colon = &line[..colon_pos];
    let value = line[colon_pos + 1..].trim().to_string();
    
    // Check for case-sensitive flag
    if before_colon.contains("case-sensitive") {
        case_sensitive = true;
    }
    
    // Parse key and operator
    let key_and_op: Vec<&str> = before_colon.split_whitespace()
        .filter(|&s| s != "or" && s != "not" && s != "case-sensitive")
        .collect();
    
    if key_and_op.len() < 2 {
        return Err(GreqError::Parse(format!("{}:{}: Invalid condition format, missing operator: '{}'", file_path, line_num, line)));
    }
    
    let key = parse_condition_key(key_and_op[0])
        .map_err(|e| GreqError::Parse(format!("{}:{}: {}", file_path, line_num, e)))?;
    let operator = parse_operator(key_and_op[1])
        .map_err(|e| GreqError::Parse(format!("{}:{}: {}", file_path, line_num, e)))?;
    
    Ok(Condition {
        is_or,
        is_not,
        key,
        operator,
        case_sensitive,
        value,
    })
}

/// Parse the header section
#[allow(dead_code)]
fn parse_header(header_text: &str) -> Result<Header> {
    let mut header = Header::default();
    
    for line in header_text.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 1..].trim();
            
            match key.as_str() {
                "project" => header.project = Some(value.to_string()),
                "is-http" => header.is_http = parse_bool(value)?,
                "delimiter" => header.delimiter = value.to_string(),
                "extends" => header.extends = Some(value.to_string()),
                "number-of-retries" => header.number_of_retries = value.parse()
                    .map_err(|_| GreqError::Parse(format!("Invalid number-of-retries: {value}")))?,
                "depends-on" => header.depends_on = Some(value.to_string()),
                "timeout" => {
                    let timeout_ms: u64 = value.parse()
                        .map_err(|_| GreqError::Parse(format!("Invalid timeout: {value}")))?;
                    header.timeout = Some(Duration::from_millis(timeout_ms));
                },
                "allow-dependency-failure" => header.allow_dependency_failure = parse_bool(value)?,
                "show-warnings" => header.show_warnings = parse_bool(value)
                    .map_err(|_| GreqError::Parse(format!("Invalid boolean value '{}' for show-warnings", value)))?,
                _ => log::warn!("Unknown header property: {key}"),
            }
        }
    }
    
    // Validate header properties
    validate_header(&header)?;
    
    Ok(header)
}

/// Validate header properties for consistency
fn validate_header(header: &Header) -> Result<()> {
    // Validate that allow-dependency-failure is only used with depends-on
    if header.allow_dependency_failure && header.depends_on.is_none() {
        return Err(GreqError::Validation(
            "allow-dependency-failure can only be used when depends-on is defined".to_string()
        ));
    }
    
    Ok(())
}

/// Parse the content section (HTTP request)
#[allow(dead_code)]
fn parse_content(content_text: &str) -> Result<Content> {
    let lines: Vec<&str> = content_text.lines().collect();
    
    if lines.is_empty() {
        return Err(GreqError::Parse("Content section cannot be empty".to_string()));
    }
    
    // Parse request line
    let request_line = parse_request_line(lines[0])?;
    
    // Parse headers and find body start
    let mut headers = HashMap::new();
    let mut body_start = lines.len();
    
    for (i, line) in lines.iter().enumerate().skip(1) {
        let line = line.trim();
        
        if line.is_empty() {
            body_start = i + 1;
            break;
        }
        
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 1..].trim().to_string();
            headers.insert(key, value);
        }
    }
    
    // Parse body
    let body = if body_start < lines.len() {
        let body_lines: Vec<&str> = lines[body_start..].to_vec();
        if body_lines.iter().any(|line| !line.trim().is_empty()) {
            Some(body_lines.join("\n"))
        } else {
            None
        }
    } else {
        None
    };
    
    Ok(Content {
        request_line,
        headers,
        body,
    })
}

/// Parse the HTTP request line
/// Parse request line
#[allow(dead_code)]
fn parse_request_line(line: &str) -> Result<RequestLine> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return Err(GreqError::Parse(format!("Invalid request line: {line}")));
    }
    
    let method = parts[0].to_uppercase();
    let uri = parts[1].to_string();
    
    // Use provided version or default to HTTP/1.1
    let version = if parts.len() >= 3 {
        parts[2].to_string()
    } else {
        "HTTP/1.1".to_string()
    };
    
    Ok(RequestLine {
        method,
        uri,
        version,
    })
}

/// Parse the footer section (conditions)
#[allow(dead_code)]
fn parse_footer(footer_text: &str) -> Result<Footer> {
    let mut conditions = Vec::new();
    
    for (line_num, line) in footer_text.lines().enumerate() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        
        match parse_condition(line) {
            Ok(condition) => conditions.push(condition),
            Err(e) => {
                return Err(GreqError::Parse(format!("Line {}: {}", line_num + 1, e)));
            }
        }
    }
    
    Ok(Footer { conditions })
}

/// Parse a single condition line
#[allow(dead_code)]
fn parse_condition(line: &str) -> Result<Condition> {
    let mut parts = line.split_whitespace().collect::<Vec<&str>>();
    let mut is_or = false;
    let mut is_not = false;
    let mut case_sensitive = false;
    
    // Check for prefixes
    if parts.first() == Some(&"or") {
        is_or = true;
        parts.remove(0);
    }
    
    if parts.first() == Some(&"not") {
        is_not = true;
        parts.remove(0);
    }
    
    if parts.len() < 3 {
        return Err(GreqError::Parse(format!("Invalid condition format: {line}")));
    }
    
    // Find colon to separate operator from value (use first colon, not last)
    let colon_pos = line.find(':')
        .ok_or_else(|| GreqError::Parse(format!("Missing colon in condition: {line}")))?;
    
    let before_colon = &line[..colon_pos];
    let value = line[colon_pos + 1..].trim().to_string();
    
    // Check for case-sensitive flag
    if before_colon.contains("case-sensitive") {
        case_sensitive = true;
    }
    
    // Parse key and operator
    let key_and_op: Vec<&str> = before_colon.split_whitespace()
        .filter(|&s| s != "or" && s != "not" && s != "case-sensitive")
        .collect();
    
    if key_and_op.len() < 2 {
        return Err(GreqError::Parse(format!("Invalid condition format: {line}")));
    }
    
    let key = parse_condition_key(key_and_op[0])?;
    let operator = parse_operator(key_and_op[1])?;
    
    Ok(Condition {
        is_or,
        is_not,
        key,
        operator,
        case_sensitive,
        value,
    })
}

/// Parse condition key
fn parse_condition_key(key_str: &str) -> Result<ConditionKey> {
    if key_str == "status-code" {
        Ok(ConditionKey::StatusCode)
    } else if key_str == "latency" {
        Ok(ConditionKey::Latency)
    } else if key_str == "response-body" {
        Ok(ConditionKey::ResponseBody)
    } else if key_str == "headers" {
        Ok(ConditionKey::Headers)
    } else if let Some(stripped) = key_str.strip_prefix("headers.") {
        let header_name = stripped.to_string();
        Ok(ConditionKey::Header(header_name))
    } else if let Some(stripped) = key_str.strip_prefix("response-body.") {
        let path = stripped.to_string();
        Ok(ConditionKey::ResponseBodyPath(path))
    } else {
        Err(GreqError::Parse(format!("Unknown condition key: {key_str}")))
    }
}

/// Parse operator
fn parse_operator(op_str: &str) -> Result<Operator> {
    match op_str {
        "equals" => Ok(Operator::Equals),
        "contains" => Ok(Operator::Contains),
        "matches-regex" => Ok(Operator::MatchesRegex),
        "less-than" => Ok(Operator::LessThan),
        "less-than-or-equal" => Ok(Operator::LessThanOrEqual),
        "greater-than" => Ok(Operator::GreaterThan),
        "greater-than-or-equal" => Ok(Operator::GreaterThanOrEqual),
        "starts-with" => Ok(Operator::StartsWith),
        "ends-with" => Ok(Operator::EndsWith),
        "exists" => Ok(Operator::Exists),
        _ => Err(GreqError::Parse(format!("Unknown operator: {op_str}"))),
    }
}

/// Parse boolean value from string
fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => Err(GreqError::Parse(format!("Invalid boolean value: {value}"))),
    }
}

/// Merge a base GreqFile with an extending GreqFile
pub fn merge_greq_files(base: &GreqFile, extending: &GreqFile) -> Result<GreqFile> {
    log::debug!("Merging base file '{}' with extending file '{}'", base.file_path, extending.file_path);
    
    let mut merged = base.clone();
    
    // Merge header properties - extending file overrides base properties
    if extending.header.project.is_some() {
        merged.header.project = extending.header.project.clone();
    }
    // For boolean fields, only override if extending file explicitly sets them differently
    if extending.header.is_http != Header::default().is_http {
        merged.header.is_http = extending.header.is_http;
    }
    if extending.header.delimiter != Header::default().delimiter {
        merged.header.delimiter = extending.header.delimiter.clone();
    }
    if extending.header.number_of_retries != Header::default().number_of_retries {
        merged.header.number_of_retries = extending.header.number_of_retries;
    }
    if extending.header.timeout.is_some() {
        merged.header.timeout = extending.header.timeout;
    }
    if extending.header.depends_on.is_some() {
        merged.header.depends_on = extending.header.depends_on.clone();
    }
    // Note: extends field is not inherited - each file manages its own extends
    
    // Merge content section
    // If extending file has content, use its request line, otherwise keep base
    // In practice, most extending files will override the request line
    if !extending.content.headers.is_empty() || extending.content.body.is_some() ||
       extending.content.request_line.method != "GET" || extending.content.request_line.uri != "/" {
        merged.content.request_line = extending.content.request_line.clone();
    }
    
    // Merge headers - start with base headers, then add/override with extending headers
    for (key, value) in &extending.content.headers {
        merged.content.headers.insert(key.clone(), value.clone());
    }
    
    // Use extending file's body if present, otherwise keep base body
    if extending.content.body.is_some() {
        merged.content.body = extending.content.body.clone();
    }
    
    // Merge footer conditions - extending file adds to or overrides base conditions
    // First, add all base conditions that don't conflict with extending conditions
    let mut merged_conditions = Vec::new();
    
    // Add all base conditions first
    for base_condition in &base.footer.conditions {
        merged_conditions.push(base_condition.clone());
    }
    
    // Add extending conditions, but replace any that have the same key
    for extending_condition in &extending.footer.conditions {
        // Remove any base condition with the same key
        merged_conditions.retain(|base_cond| !conditions_have_same_key(base_cond, extending_condition));
        // Add the extending condition
        merged_conditions.push(extending_condition.clone());
    }
    
    merged.footer.conditions = merged_conditions;
    merged.file_path = extending.file_path.clone();
    
    Ok(merged)
}

/// Check if two conditions have the same key (for merging purposes)
fn conditions_have_same_key(cond1: &Condition, cond2: &Condition) -> bool {
    std::mem::discriminant(&cond1.key) == std::mem::discriminant(&cond2.key) &&
    match (&cond1.key, &cond2.key) {
        (ConditionKey::Header(h1), ConditionKey::Header(h2)) => h1 == h2,
        (ConditionKey::ResponseBodyPath(p1), ConditionKey::ResponseBodyPath(p2)) => p1 == p2,
        _ => true, // For non-parameterized keys, they're the same if discriminants match
    }
}

/// Resolve file path relative to current file
pub fn resolve_file_path<P: AsRef<Path>>(current_file: P, referenced_file: &str) -> PathBuf {
    let current_dir = current_file.as_ref().parent().unwrap_or(Path::new("."));
    
    // Add .greq extension if not present
    let file_name = if referenced_file.ends_with(".greq") {
        referenced_file.to_string()
    } else {
        format!("{referenced_file}.greq")
    };
    
    if Path::new(&file_name).is_absolute() {
        PathBuf::from(file_name)
    } else {
        current_dir.join(file_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_header() {
        let header_text = r#"
project: test project
is-http: true
number-of-retries: 3
timeout: 5000
"#;
        
        let header = parse_header(header_text).unwrap();
        assert_eq!(header.project, Some("test project".to_string()));
        assert_eq!(header.is_http, true);
        assert_eq!(header.number_of_retries, 3);
        assert_eq!(header.timeout, Some(Duration::from_millis(5000)));
    }
    
    #[test]
    fn test_parse_request_line() {
        let line = "POST /api/test HTTP/1.1";
        let request_line = parse_request_line(line).unwrap();
        
        assert_eq!(request_line.method, "POST");
        assert_eq!(request_line.uri, "/api/test");
        assert_eq!(request_line.version, "HTTP/1.1");
    }
    
    #[test]
    fn test_parse_condition() {
        let line = "status-code equals: 200";
        let condition = parse_condition(line).unwrap();
        
        assert!(!condition.is_or);
        assert!(!condition.is_not);
        assert!(matches!(condition.key, ConditionKey::StatusCode));
        assert!(matches!(condition.operator, Operator::Equals));
        assert_eq!(condition.value, "200");
    }
    
    #[test]
    fn test_parse_condition_with_prefixes() {
        let line = "or not response-body contains case-sensitive: Success";
        let condition = parse_condition(line).unwrap();
        
        assert!(condition.is_or);
        assert!(condition.is_not);
        assert!(matches!(condition.key, ConditionKey::ResponseBody));
        assert!(matches!(condition.operator, Operator::Contains));
        assert!(condition.case_sensitive);
        assert_eq!(condition.value, "Success");
    }

    #[test]
    fn test_parse_header_with_allow_dependency_failure() {
        let header_text = r#"
project: test project
depends-on: auth.greq
allow-dependency-failure: true
"#;
        
        let header = parse_header(header_text).unwrap();
        assert_eq!(header.project, Some("test project".to_string()));
        assert_eq!(header.depends_on, Some("auth.greq".to_string()));
        assert_eq!(header.allow_dependency_failure, true);
    }
    
    #[test]
    fn test_validate_header_allow_dependency_failure_without_depends_on() {
        let header = Header {
            project: Some("test".to_string()),
            allow_dependency_failure: true,
            depends_on: None,
            ..Header::default()
        };
        
        let result = validate_header(&header);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("allow-dependency-failure can only be used when depends-on is defined"));
    }
    
    #[test]
    fn test_validate_header_allow_dependency_failure_with_depends_on() {
        let header = Header {
            project: Some("test".to_string()),
            allow_dependency_failure: true,
            depends_on: Some("auth.greq".to_string()),
            ..Header::default()
        };
        
        let result = validate_header(&header);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_header_without_allow_dependency_failure() {
        let header = Header {
            project: Some("test".to_string()),
            allow_dependency_failure: false,
            depends_on: None,
            ..Header::default()
        };
        
        let result = validate_header(&header);
        assert!(result.is_ok());
    }

    #[test]
    fn test_condition_parsing_with_multiple_colons_in_value() {
        let content = r#"
project: test project
is-http: true

====

GET /test HTTP/1.1
host: httpbin.org

====

response-body.domain.domain_url equals: https://greq-test-grauth.me
status-code equals: 200
response-body contains: https://example.com:8080/path?param=value
response-body.api_url equals: http://localhost:3000/api/v1/users?filter=active
"#;

        let result = parse_greq_content(content, "test.greq");
        assert!(result.is_ok(), "Failed to parse file with URL values: {:?}", result.err());
        
        let greq_file = result.unwrap();
        assert_eq!(greq_file.footer.conditions.len(), 4);
        
        // Check the URL condition with domain path
        let url_condition = &greq_file.footer.conditions[0];
        assert_eq!(url_condition.value, "https://greq-test-grauth.me");
        assert!(matches!(url_condition.key, ConditionKey::ResponseBodyPath(_)));
        assert!(matches!(url_condition.operator, Operator::Equals));
        
        // Check the contains condition with port and path
        let contains_condition = &greq_file.footer.conditions[2];
        assert_eq!(contains_condition.value, "https://example.com:8080/path?param=value");
        assert!(matches!(contains_condition.key, ConditionKey::ResponseBody));
        assert!(matches!(contains_condition.operator, Operator::Contains));
        
        // Check the complex API URL
        let api_condition = &greq_file.footer.conditions[3];
        assert_eq!(api_condition.value, "http://localhost:3000/api/v1/users?filter=active");
        assert!(matches!(api_condition.key, ConditionKey::ResponseBodyPath(_)));
        assert!(matches!(api_condition.operator, Operator::Equals));
    }

    #[test]
    fn test_parsing_error_shows_line_number_and_file() {
        let content = r#"
project: test project
is-http: true

====

GET /test HTTP/1.1
host: httpbin.org

====

status-code invalid-operator: 200
response-body equals: success
"#;

        let result = parse_greq_content(content, "test.greq");
        assert!(result.is_err());
        
        let error = result.err().unwrap();
        let error_msg = format!("{}", error);
        
        // Should contain file name and line number
        assert!(error_msg.contains("test.greq"), "Error should contain file name: {}", error_msg);
        assert!(error_msg.contains(":12:"), "Error should contain line number 12: {}", error_msg);
        assert!(error_msg.contains("invalid-operator"), "Error should mention the invalid operator: {}", error_msg);
    }

    #[test]
    fn test_parsing_error_missing_colon_in_condition() {
        let content = r#"
project: test project

====

GET /test HTTP/1.1
host: httpbin.org

====

status-code equals 200
"#;

        let result = parse_greq_content(content, "missing-colon.greq");
        assert!(result.is_err());
        
        let error = result.err().unwrap();
        let error_msg = format!("{}", error);
        
        assert!(error_msg.contains("missing-colon.greq"), "Error should contain file name: {}", error_msg);
        assert!(error_msg.contains(":11:"), "Error should contain line number 11: {}", error_msg);
        assert!(error_msg.contains("Missing colon"), "Error should mention missing colon: {}", error_msg);
    }

    #[test]
    fn test_parsing_error_in_header_shows_line_number() {
        let content = r#"
project: test project
invalid-property without-colon

====

GET /test HTTP/1.1
host: httpbin.org
"#;

        let result = parse_greq_content(content, "header-error.greq");
        assert!(result.is_err());
        
        let error = result.err().unwrap();
        let error_msg = format!("{}", error);
        
        assert!(error_msg.contains("header-error.greq"), "Error should contain file name: {}", error_msg);
        assert!(error_msg.contains(":2:"), "Error should contain line number 2: {}", error_msg);
        assert!(error_msg.contains("Missing colon"), "Error should mention missing colon: {}", error_msg);
    }

    #[test]
    fn test_complex_url_conditions() {
        let content = r#"
project: URL test with complex scenarios

====

GET /test HTTP/1.1
host: example.com

====

response-body.api.endpoint equals: https://api.example.com:443/v1/users?filter=active&sort=name
response-body.callback_url equals: http://localhost:3000/webhook?token=abc123&event=user.created
headers.location contains: https://secure.example.com:8443/redirect?target=dashboard
response-body.websocket_url equals: wss://ws.example.com:8080/stream?auth=bearer-token
response-body.database_url equals: postgresql://user:pass@localhost:5432/dbname?sslmode=require
"#;

        let result = parse_greq_content(content, "url-test.greq");
        assert!(result.is_ok(), "Failed to parse file with complex URLs: {:?}", result.err());
        
        let greq_file = result.unwrap();
        assert_eq!(greq_file.footer.conditions.len(), 5);
        
        // Check complex API URL with query parameters
        let api_condition = &greq_file.footer.conditions[0];
        assert_eq!(api_condition.value, "https://api.example.com:443/v1/users?filter=active&sort=name");
        
        // Check callback URL with query parameters
        let callback_condition = &greq_file.footer.conditions[1];
        assert_eq!(callback_condition.value, "http://localhost:3000/webhook?token=abc123&event=user.created");
        
        // Check secure URL in header check
        let location_condition = &greq_file.footer.conditions[2];
        assert_eq!(location_condition.value, "https://secure.example.com:8443/redirect?target=dashboard");
        
        // Check WebSocket URL
        let websocket_condition = &greq_file.footer.conditions[3];
        assert_eq!(websocket_condition.value, "wss://ws.example.com:8080/stream?auth=bearer-token");
        
        // Check database URL with credentials and parameters
        let db_condition = &greq_file.footer.conditions[4];
        assert_eq!(db_condition.value, "postgresql://user:pass@localhost:5432/dbname?sslmode=require");
    }

    #[test]
    fn test_parse_header_with_show_warnings() {
        let content = "show-warnings: false\nproject: Test Project\n";
        let result = parse_header_with_lines(content, "test.greq", 1);
        
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.show_warnings, false);
        assert_eq!(header.project, Some("Test Project".to_string()));
    }

    #[test]
    fn test_parse_header_show_warnings_default() {
        let content = "project: Test Project\n";
        let result = parse_header_with_lines(content, "test.greq", 1);
        
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.show_warnings, true); // default value
        assert_eq!(header.project, Some("Test Project".to_string()));
    }

    #[test]
    fn test_parse_header_invalid_show_warnings() {
        let content = "show-warnings: invalid\nproject: Test Project\n";
        let result = parse_header_with_lines(content, "test.greq", 1);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid boolean value 'invalid' for show-warnings"));
    }
}
