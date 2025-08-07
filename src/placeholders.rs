use crate::models::Response;
use crate::error::{GreqError, Result};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// Replace placeholders in a string with values from dependency response
pub fn replace_placeholders(text: &str, dependency_response: &Response) -> Result<String> {
    let placeholder_regex = Regex::new(r"\$\((dep(?:endency)?\.[\w\.\[\]-]+)\)")?;
    let mut result = text.to_string();
    
    for capture in placeholder_regex.captures_iter(text) {
        let full_match = &capture[0];
        let placeholder_path = &capture[1];
        
        let value = extract_value_from_response(placeholder_path, dependency_response)?;
        result = result.replace(full_match, &value);
    }
    
    Ok(result)
}

/// Extract a value from response based on placeholder path
fn extract_value_from_response(path: &str, response: &Response) -> Result<String> {
    log::debug!("Extracting value for path: {}", path);
    
    // Remove 'dependency.' or 'dep.' prefix
    let path = if path.starts_with("dependency.") {
        &path[11..]
    } else if path.starts_with("dep.") {
        &path[4..]
    } else {
        return Err(GreqError::Placeholder(format!("Invalid placeholder path: {}", path)));
    };
    
    match path {
        "status-code" => Ok(response.status_code.to_string()),
        "latency" => Ok(response.latency.as_millis().to_string()),
        "headers" => {
            let headers_json = serde_json::to_string(&response.headers)?;
            Ok(headers_json)
        },
        "response-body" => Ok(response.body.clone()),
        _ => {
            if path.starts_with("headers.") {
                let header_name = &path[8..].to_lowercase();
                Ok(response.headers.get(header_name).cloned().unwrap_or_default())
            } else if path.starts_with("response-body.") {
                let json_path = &path[14..];
                extract_json_path(&response.body, json_path)
            } else {
                Err(GreqError::Placeholder(format!("Unknown placeholder path: {}", path)))
            }
        }
    }
}

/// Extract value from JSON using JSONPath-like syntax
fn extract_json_path(json_text: &str, path: &str) -> Result<String> {
    let value: Value = serde_json::from_str(json_text)
        .map_err(|_| GreqError::Placeholder("Response body is not valid JSON".to_string()))?;
    
    let result = navigate_json_path(&value, path)?;
    
    match result {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok("null".to_string()),
        _ => Ok(serde_json::to_string(&result)?),
    }
}

/// Navigate through JSON using a simple path syntax
fn navigate_json_path(value: &Value, path: &str) -> Result<Value> {
    let mut current = value;
    let parts = parse_json_path(path)?;
    
    for part in parts {
        match part {
            PathPart::Property(key) => {
                if let Value::Object(obj) = current {
                    current = obj.get(&key)
                        .ok_or_else(|| GreqError::Placeholder(format!("Property '{}' not found", key)))?;
                } else {
                    return Err(GreqError::Placeholder(format!("Cannot access property '{}' on non-object", key)));
                }
            },
            PathPart::Index(index) => {
                if let Value::Array(arr) = current {
                    current = arr.get(index)
                        .ok_or_else(|| GreqError::Placeholder(format!("Array index {} out of bounds", index)))?;
                } else {
                    return Err(GreqError::Placeholder(format!("Cannot access index {} on non-array", index)));
                }
            },
        }
    }
    
    Ok(current.clone())
}

/// Represents a part of a JSON path
#[derive(Debug, Clone)]
enum PathPart {
    Property(String),
    Index(usize),
}

/// Parse a JSON path string into path parts
fn parse_json_path(path: &str) -> Result<Vec<PathPart>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !current.is_empty() {
                    parts.push(PathPart::Property(current.clone()));
                    current.clear();
                }
            },
            '[' => {
                if !current.is_empty() {
                    parts.push(PathPart::Property(current.clone()));
                    current.clear();
                }
                
                // Parse array index
                let mut index_str = String::new();
                while let Some(ch) = chars.next() {
                    if ch == ']' {
                        break;
                    }
                    index_str.push(ch);
                }
                
                let index: usize = index_str.parse()
                    .map_err(|_| GreqError::Placeholder(format!("Invalid array index: {}", index_str)))?;
                parts.push(PathPart::Index(index));
            },
            _ => {
                current.push(ch);
            }
        }
    }
    
    if !current.is_empty() {
        parts.push(PathPart::Property(current));
    }
    
    Ok(parts)
}

/// Replace placeholders in all text fields of a GreqFile
pub fn replace_placeholders_in_greq_file(
    greq_file: &mut crate::models::GreqFile,
    dependency_response: &Response,
) -> Result<()> {
    // Replace in URI
    greq_file.content.request_line.uri = replace_placeholders(
        &greq_file.content.request_line.uri,
        dependency_response,
    )?;
    
    // Replace in headers
    let mut updated_headers = HashMap::new();
    for (key, value) in &greq_file.content.headers {
        let updated_value = replace_placeholders(value, dependency_response)?;
        updated_headers.insert(key.clone(), updated_value);
    }
    greq_file.content.headers = updated_headers;
    
    // Replace in body
    if let Some(body) = &greq_file.content.body {
        greq_file.content.body = Some(replace_placeholders(body, dependency_response)?);
    }
    
    // Replace in condition values
    for condition in &mut greq_file.footer.conditions {
        condition.value = replace_placeholders(&condition.value, dependency_response)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    fn create_test_response() -> Response {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        
        Response {
            status_code: 200,
            headers,
            body: r#"{"id": 123, "name": "test", "items": [{"id": 1}, {"id": 2}]}"#.to_string(),
            latency: Duration::from_millis(150),
        }
    }
    
    #[test]
    fn test_replace_status_code_placeholder() {
        let response = create_test_response();
        let text = "Status: $(dependency.status-code)";
        let result = replace_placeholders(text, &response).unwrap();
        assert_eq!(result, "Status: 200");
    }
    
    #[test]
    fn test_replace_header_placeholder() {
        let response = create_test_response();
        let text = "Type: $(dependency.headers.content-type)";
        let result = replace_placeholders(text, &response).unwrap();
        assert_eq!(result, "Type: application/json");
    }
    
    #[test]
    fn test_replace_json_path_placeholder() {
        let response = create_test_response();
        let text = "ID: $(dependency.response-body.id)";
        let result = replace_placeholders(text, &response).unwrap();
        assert_eq!(result, "ID: 123");
    }
    
    #[test]
    fn test_replace_json_array_placeholder() {
        let response = create_test_response();
        let text = "First item: $(dependency.response-body.items[0].id)";
        let result = replace_placeholders(text, &response).unwrap();
        assert_eq!(result, "First item: 1");
    }
    
    #[test]
    fn test_parse_json_path() {
        let parts = parse_json_path("items[0].id").unwrap();
        assert_eq!(parts.len(), 3);
        
        match &parts[0] {
            PathPart::Property(name) => assert_eq!(name, "items"),
            _ => panic!("Expected property"),
        }
        
        match &parts[1] {
            PathPart::Index(index) => assert_eq!(*index, 0),
            _ => panic!("Expected index"),
        }
        
        match &parts[2] {
            PathPart::Property(name) => assert_eq!(name, "id"),
            _ => panic!("Expected property"),
        }
    }
}
