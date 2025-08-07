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
    log::debug!("Parsing greq file: {:?}", file_path);
    
    let content = fs::read_to_string(file_path)
        .map_err(|_| GreqError::FileNotFound(file_path.display().to_string()))?;
    
    let file_path_str = file_path.display().to_string();
    let sections = split_into_sections(&content, "=")?;
    
    if sections.len() < 2 {
        return Err(GreqError::Parse("Invalid file format: must have at least header and content sections".to_string()));
    }
    
    let header = parse_header(&sections[0])?;
    let delimiter = header.delimiter.clone();
    
    // Re-split with custom delimiter if specified
    let sections = if delimiter != "=" {
        split_into_sections(&content, &delimiter)?
    } else {
        sections
    };
    
    let content_section = parse_content(&sections[1])?;
    let footer = if sections.len() > 2 {
        parse_footer(&sections[2])?
    } else {
        Footer::default()
    };
    
    Ok(GreqFile {
        header,
        content: content_section,
        footer,
        file_path: file_path_str,
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
        log::debug!("Section {}: {:?}", i, section);
    }
    
    if sections.len() < 2 {
        return Err(GreqError::Parse("File must contain at least 2 sections separated by delimiter".to_string()));
    }
    
    Ok(sections)
}

/// Parse the header section
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
                    .map_err(|_| GreqError::Parse(format!("Invalid number-of-retries: {}", value)))?,
                "depends-on" => header.depends_on = Some(value.to_string()),
                "timeout" => {
                    let timeout_ms: u64 = value.parse()
                        .map_err(|_| GreqError::Parse(format!("Invalid timeout: {}", value)))?;
                    header.timeout = Some(Duration::from_millis(timeout_ms));
                },
                _ => log::warn!("Unknown header property: {}", key),
            }
        }
    }
    
    Ok(header)
}

/// Parse the content section (HTTP request)
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
        let body_lines: Vec<&str> = lines[body_start..].iter().cloned().collect();
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
fn parse_request_line(line: &str) -> Result<RequestLine> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 3 {
        return Err(GreqError::Parse(format!("Invalid request line: {}", line)));
    }
    
    Ok(RequestLine {
        method: parts[0].to_uppercase(),
        uri: parts[1].to_string(),
        version: parts[2].to_string(),
    })
}

/// Parse the footer section (conditions)
fn parse_footer(footer_text: &str) -> Result<Footer> {
    let mut conditions = Vec::new();
    
    for line in footer_text.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        
        conditions.push(parse_condition(line)?);
    }
    
    Ok(Footer { conditions })
}

/// Parse a single condition line
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
        return Err(GreqError::Parse(format!("Invalid condition format: {}", line)));
    }
    
    // Find colon to separate operator from value
    let colon_pos = line.rfind(':')
        .ok_or_else(|| GreqError::Parse(format!("Missing colon in condition: {}", line)))?;
    
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
        return Err(GreqError::Parse(format!("Invalid condition format: {}", line)));
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
    } else if key_str.starts_with("headers.") {
        let header_name = key_str[8..].to_string();
        Ok(ConditionKey::Header(header_name))
    } else if key_str.starts_with("response-body.") {
        let path = key_str[14..].to_string();
        Ok(ConditionKey::ResponseBodyPath(path))
    } else {
        Err(GreqError::Parse(format!("Unknown condition key: {}", key_str)))
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
        _ => Err(GreqError::Parse(format!("Unknown operator: {}", op_str))),
    }
}

/// Parse boolean value from string
fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => Err(GreqError::Parse(format!("Invalid boolean value: {}", value))),
    }
}

/// Merge a base GreqFile with an extending GreqFile
pub fn merge_greq_files(base: &GreqFile, extending: &GreqFile) -> Result<GreqFile> {
    log::debug!("Merging base file '{}' with extending file '{}'", base.file_path, extending.file_path);
    
    let mut merged = base.clone();
    
    // Override header properties if specified in extending file
    if extending.header.project.is_some() {
        merged.header.project = extending.header.project.clone();
    }
    if extending.header.is_http != base.header.is_http {
        merged.header.is_http = extending.header.is_http;
    }
    if extending.header.delimiter != base.header.delimiter {
        merged.header.delimiter = extending.header.delimiter.clone();
    }
    if extending.header.number_of_retries != base.header.number_of_retries {
        merged.header.number_of_retries = extending.header.number_of_retries;
    }
    if extending.header.timeout.is_some() {
        merged.header.timeout = extending.header.timeout;
    }
    if extending.header.depends_on.is_some() {
        merged.header.depends_on = extending.header.depends_on.clone();
    }
    
    // Always use the extending file's request line (required)
    merged.content.request_line = extending.content.request_line.clone();
    
    // Merge headers (extending overrides base)
    for (key, value) in &extending.content.headers {
        merged.content.headers.insert(key.clone(), value.clone());
    }
    
    // Use extending file's body if present
    if extending.content.body.is_some() {
        merged.content.body = extending.content.body.clone();
    }
    
    // Use extending file's conditions if present
    if !extending.footer.conditions.is_empty() {
        merged.footer = extending.footer.clone();
    }
    
    merged.file_path = extending.file_path.clone();
    
    Ok(merged)
}

/// Resolve file path relative to current file
pub fn resolve_file_path<P: AsRef<Path>>(current_file: P, referenced_file: &str) -> PathBuf {
    let current_dir = current_file.as_ref().parent().unwrap_or(Path::new("."));
    
    // Add .greq extension if not present
    let file_name = if referenced_file.ends_with(".greq") {
        referenced_file.to_string()
    } else {
        format!("{}.greq", referenced_file)
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
}
