use crate::models::Response;
use crate::error::{GreqError, Result};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::env;

/// Replace placeholders in a string with values from dependency response or environment variables
pub fn replace_placeholders(text: &str, dependency_response: &Response) -> Result<String> {
    replace_placeholders_with_context(text, dependency_response, "unknown", "unknown location")
}

/// Replace placeholders with file context for better error reporting
pub fn replace_placeholders_with_context(
    text: &str, 
    dependency_response: &Response, 
    file_path: &str, 
    location: &str
) -> Result<String> {
    let placeholder_regex = Regex::new(r"\$\(([\w\.\-\[\]]+)\)")?;
    let mut result = text.to_string();
    
    for capture in placeholder_regex.captures_iter(text) {
        let full_match = &capture[0];
        let placeholder_path = &capture[1];
        
        let value = if placeholder_path.starts_with("environment.") {
            extract_environment_variable_with_context(placeholder_path, file_path, location)?
        } else {
            extract_value_from_response_with_context(placeholder_path, dependency_response, file_path, location)?
        };
        
        result = result.replace(full_match, &value);
    }
    
    Ok(result)
}

/// Extract environment variable value from placeholder path with context
fn extract_environment_variable_with_context(
    path: &str, 
    file_path: &str, 
    location: &str
) -> Result<String> {
    let env_var_name = path.strip_prefix("environment.")
        .ok_or_else(|| GreqError::Placeholder(format!("{}: {}: Invalid environment placeholder: {path}", file_path, location)))?;
    
    if env_var_name.is_empty() {
        return Err(GreqError::Placeholder(format!("{}: {}: Environment variable name cannot be empty", file_path, location)));
    }
    
    log::debug!("Extracting environment variable: {env_var_name}");
    
    env::var(env_var_name)
        .map_err(|_| GreqError::Placeholder(format!("{}: {}: Environment variable '{env_var_name}' not found", file_path, location)))
}

/// Extract a value from response based on placeholder path with context
fn extract_value_from_response_with_context(
    path: &str, 
    response: &Response, 
    file_path: &str, 
    location: &str
) -> Result<String> {
    log::debug!("Extracting value for path: {path}");
    
    // Handle dependency-based paths like "dependency.status-code" or "dep.response-body.user.id"
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return Err(GreqError::Placeholder(format!("{}: {}: Invalid placeholder path: {path}", file_path, location)));
    }
    
    // Check if it starts with dependency prefix
    let actual_path = if parts[0] == "dependency" || parts[0] == "dep" {
        if parts.len() < 2 {
            return Err(GreqError::Placeholder(format!("{}: {}: Invalid dependency placeholder path: {path}", file_path, location)));
        }
        parts[1..].join(".")
    } else {
        // Handle legacy file-name-based paths for backward compatibility
        if parts.len() < 2 {
            return Err(GreqError::Placeholder(format!("{}: {}: Invalid placeholder path: {path}", file_path, location)));
        }
        parts[1..].join(".")
    };
    
    match actual_path.as_str() {
        "status-code" => Ok(response.status_code.to_string()),
        "latency" => Ok(response.latency.as_millis().to_string()),
        "headers" => {
            let headers_json = serde_json::to_string(&response.headers)?;
            Ok(headers_json)
        },
        "response-body" => Ok(response.body.clone()),
        _ => {
            if let Some(stripped) = actual_path.strip_prefix("headers.") {
                let header_name = &stripped.to_lowercase();
                Ok(response.headers.get(header_name).cloned().unwrap_or_default())
            } else if let Some(json_path) = actual_path.strip_prefix("response-body.") {
                extract_json_path_with_context(&response.body, json_path, file_path, location)
            } else {
                Err(GreqError::Placeholder(format!("{}: {}: Unknown placeholder path: {actual_path}", file_path, location)))
            }
        }
    }
}

/// Extract value from JSON using JSONPath-like syntax with context
fn extract_json_path_with_context(
    json_text: &str, 
    path: &str, 
    file_path: &str, 
    location: &str
) -> Result<String> {
    let value: Value = serde_json::from_str(json_text)
        .map_err(|_| GreqError::Placeholder(format!("{}: {}: Response body is not valid JSON", file_path, location)))?;
    
    let result = navigate_json_path_with_context(&value, path, file_path, location)?;
    
    match result {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok("null".to_string()),
        _ => Ok(serde_json::to_string(&result)?),
    }
}

/// Navigate through JSON using a simple path syntax with context
fn navigate_json_path_with_context(
    value: &Value, 
    path: &str, 
    file_path: &str, 
    location: &str
) -> Result<Value> {
    let mut current = value;
    let parts = parse_json_path(path)?;
    
    for part in parts {
        match part {
            PathPart::Property(key) => {
                if let Value::Object(obj) = current {
                    current = obj.get(&key)
                        .ok_or_else(|| GreqError::Placeholder(format!("{}: {}: Property '{}' not found in JSON path '{}'. Please check the actual response.", file_path, location, key, path)))?;
                } else {
                    return Err(GreqError::Placeholder(format!("{}: {}: Cannot access property '{}' on non-object in JSON path '{}'", file_path, location, key, path)));
                }
            },
            PathPart::Index(index) => {
                if let Value::Array(arr) = current {
                    current = arr.get(index)
                        .ok_or_else(|| GreqError::Placeholder(format!("{}: {}: Array index {} out of bounds (length: {}) in JSON path '{}'", file_path, location, index, arr.len(), path)))?;
                } else {
                    return Err(GreqError::Placeholder(format!("{}: {}: Cannot access index {} on non-array in JSON path '{}'", file_path, location, index, path)));
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
                for ch in chars.by_ref() {
                    if ch == ']' {
                        break;
                    }
                    index_str.push(ch);
                }
                
                let index: usize = index_str.parse()
                    .map_err(|_| GreqError::Placeholder(format!("Invalid array index: {index_str}")))?;
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
    replace_placeholders_in_greq_file_with_optional_response(greq_file, Some(dependency_response))
}

/// Replace placeholders in all text fields of a GreqFile, with optional dependency response
pub fn replace_placeholders_in_greq_file_with_optional_response(
    greq_file: &mut crate::models::GreqFile,
    dependency_response: Option<&Response>,
) -> Result<()> {
    replace_placeholders_in_greq_file_with_dependency_handling(
        greq_file,
        dependency_response,
        false, // dependency_failed
    )
}

/// Replace dependency placeholders with empty string when dependency fails
fn replace_dependency_placeholders_with_empty_string(
    text: &str,
    file_path: &str,
    location: &str,
    _should_warn: bool,
) -> Result<String> {
    let mut result = text.to_string();
    let placeholder_regex = regex::Regex::new(r"\$\(([^)]+)\)").unwrap();
    
    // Find and replace only dependency placeholders
    for cap in placeholder_regex.find_iter(text) {
        let placeholder = cap.as_str();
        let path = &placeholder[2..placeholder.len()-1]; // Remove $( and )
        
        if path.starts_with("dependency.") || path.starts_with("dep.") {
            // Replace dependency placeholder with empty string
            result = result.replace(placeholder, "");
        } else if path.starts_with("environment.") {
            // Keep environment placeholders - they should still work
            let env_result = extract_environment_variable_with_context(path, file_path, location)?;
            result = result.replace(placeholder, &env_result);
        } else {
            // Unknown placeholder type - replace with empty string to avoid errors
            result = result.replace(placeholder, "");
        }
    }
    
    Ok(result)
}

/// Validate that dependency placeholders are only used when depends-on is provided
fn validate_dependency_placeholders(
    greq_file: &crate::models::GreqFile,
) -> Result<()> {
    if greq_file.header.depends_on.is_some() {
        return Ok(()); // Has dependency, validation passes
    }
    
    let placeholder_regex = regex::Regex::new(r"\$\(([^)]+)\)").unwrap();
    let file_path = &greq_file.file_path;
    
    // Check all text fields for dependency placeholders
    
    // Check headers
    for (header_name, header_value) in &greq_file.content.headers {
        for cap in placeholder_regex.captures_iter(header_value) {
            let path = &cap[1];
            if path.starts_with("dependency.") || path.starts_with("dep.") {
                return Err(GreqError::Validation(format!(
                    "{}: header '{}': Dependency placeholder '{}' found but no 'depends-on' is defined",
                    file_path, header_name, path
                )));
            }
        }
    }
    
    // Check request body
    if let Some(body) = &greq_file.content.body {
        for cap in placeholder_regex.captures_iter(body) {
            let path = &cap[1];
            if path.starts_with("dependency.") || path.starts_with("dep.") {
                return Err(GreqError::Validation(format!(
                    "{}: request body: Dependency placeholder '{}' found but no 'depends-on' is defined",
                    file_path, path
                )));
            }
        }
    }
    
    // Check URI
    for cap in placeholder_regex.captures_iter(&greq_file.content.request_line.uri) {
        let path = &cap[1];
        if path.starts_with("dependency.") || path.starts_with("dep.") {
            return Err(GreqError::Validation(format!(
                "{}: request URI: Dependency placeholder '{}' found but no 'depends-on' is defined",
                file_path, path
            )));
        }
    }
    
    Ok(())
}

/// Enhanced version that handles dependency failures and shows warnings
pub fn replace_placeholders_in_greq_file_with_dependency_handling(
    greq_file: &mut crate::models::GreqFile,
    dependency_response: Option<&Response>,
    dependency_failed: bool,
) -> Result<()> {
    // Validate that dependency placeholders are only used when depends-on is provided
    validate_dependency_placeholders(greq_file)?;
    
    // Check if we should show warnings and if dependency failure with placeholders should warn
    let should_warn = greq_file.header.show_warnings 
        && greq_file.header.allow_dependency_failure 
        && greq_file.header.depends_on.is_some() 
        && dependency_failed;
    
    // Create a dummy response if none provided (for environment-only placeholders)
    let dummy_response = Response {
        status_code: 200,
        headers: HashMap::new(),
        body: "{}".to_string(),
        latency: std::time::Duration::from_millis(0),
    };
    
    let response = dependency_response.unwrap_or(&dummy_response);
    let file_path = &greq_file.file_path;
    
    let mut placeholder_warning_shown = false;
    
    // Helper function to replace placeholders and handle warnings
    let replace_with_warning = |text: &str, location: &str, warning_shown: &mut bool| -> Result<String> {
        if dependency_failed && greq_file.header.allow_dependency_failure {
            // For dependency failures with allow-dependency-failure, replace dependency placeholders with empty string
            let result = replace_dependency_placeholders_with_empty_string(text, file_path, location, should_warn && !*warning_shown)?;
            if should_warn && !*warning_shown && result != *text {
                // Show warning only for the first placeholder encountered
                log::warn!("\x1b[33m⚠ Warning: {}: {}: Dependency placeholder found but dependency failed. Placeholder will be replaced with empty string.\x1b[0m", file_path, location);
                *warning_shown = true;
            }
            Ok(result)
        } else {
            // Normal replacement
            replace_placeholders_with_context(text, response, file_path, location)
        }
    };
    
    // Replace in URI
    greq_file.content.request_line.uri = replace_with_warning(
        &greq_file.content.request_line.uri,
        "request URI",
        &mut placeholder_warning_shown,
    )?;
    
    // Replace in headers
    let mut updated_headers = HashMap::new();
    for (key, value) in &greq_file.content.headers {
        let updated_value = replace_with_warning(
            value,
            &format!("header '{}'", key),
            &mut placeholder_warning_shown,
        )?;
        updated_headers.insert(key.clone(), updated_value);
    }
    greq_file.content.headers = updated_headers;
    
    // Replace in body
    if let Some(body) = &greq_file.content.body {
        greq_file.content.body = Some(replace_with_warning(
            body,
            "request body",
            &mut placeholder_warning_shown,
        )?);
    }
    
    // Replace in condition values
    for (i, condition) in greq_file.footer.conditions.iter_mut().enumerate() {
        condition.value = replace_with_warning(
            &condition.value,
            &format!("condition {} value", i + 1),
            &mut placeholder_warning_shown,
        )?;
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
    fn test_replace_environment_variable_placeholder() {
        let response = create_test_response();
        
        // Set a test environment variable
        env::set_var("TEST_VAR", "test_value");
        
        let text = "API Key: $(environment.TEST_VAR)";
        let result = replace_placeholders(text, &response).unwrap();
        assert_eq!(result, "API Key: test_value");
        
        // Clean up
        env::remove_var("TEST_VAR");
    }
    
    #[test]
    fn test_replace_environment_variable_with_underscores() {
        let response = create_test_response();
        
        // Set a test environment variable with underscores
        env::set_var("MY_API_KEY", "secret123");
        
        let text = "Token: $(environment.MY_API_KEY)";
        let result = replace_placeholders(text, &response).unwrap();
        assert_eq!(result, "Token: secret123");
        
        // Clean up
        env::remove_var("MY_API_KEY");
    }
    
    #[test]
    fn test_replace_nonexistent_environment_variable() {
        let response = create_test_response();
        
        let text = "Value: $(environment.NONEXISTENT_VAR)";
        let result = replace_placeholders(text, &response);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Environment variable 'NONEXISTENT_VAR' not found"));
    }
    
    #[test]
    fn test_replace_empty_environment_variable_name() {
        let response = create_test_response();
        
        let text = "Value: $(environment.)";
        let result = replace_placeholders(text, &response);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Environment variable name cannot be empty"));
    }
    
    #[test]
    fn test_replace_mixed_placeholders() {
        let response = create_test_response();
        
        // Set a test environment variable
        env::set_var("TEST_HOST", "api.example.com");
        
        let text = "URL: https://$(environment.TEST_HOST)/users/$(dependency.response-body.id)";
        let result = replace_placeholders(text, &response).unwrap();
        assert_eq!(result, "URL: https://api.example.com/users/123");
        
        // Clean up
        env::remove_var("TEST_HOST");
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

    #[test]
    fn test_placeholder_error_with_context() {
        use std::collections::HashMap;
        use crate::models::{GreqFile, Header, Content, RequestLine, Footer};

        let mut greq_file = GreqFile {
            header: Header {
                project: Some("Test".to_string()),
                is_http: true,
                delimiter: "====".to_string(),
                extends: None,
                number_of_retries: 0,
                depends_on: None,
                timeout: None,
                allow_dependency_failure: false,
                show_warnings: true,
            },
            content: Content {
                request_line: RequestLine {
                    method: "GET".to_string(),
                    uri: "/".to_string(),
                    version: "HTTP/1.1".to_string(),
                },
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("authorization".to_string(), "Bearer $(nonexistent.token)".to_string());
                    headers
                },
                body: None,
            },
            footer: Footer::default(),
            file_path: "test-file.greq".to_string(),
        };

        let result = replace_placeholders_in_greq_file_with_optional_response(&mut greq_file, None);

        assert!(result.is_err());
        if let Err(error) = result {
            let error_message = error.to_string();
            assert!(error_message.contains("test-file.greq"));
            assert!(error_message.contains("header 'authorization'"));
            assert!(error_message.contains("Unknown placeholder path: token"));
        }
    }

    #[test]
    fn test_environment_variable_error_with_context() {
        use std::collections::HashMap;
        use crate::models::{GreqFile, Header, Content, RequestLine, Footer};

        let mut greq_file = GreqFile {
            header: Header {
                project: Some("Test".to_string()),
                is_http: true,
                delimiter: "====".to_string(),
                extends: None,
                number_of_retries: 0,
                depends_on: None,
                timeout: None,
                allow_dependency_failure: false,
                show_warnings: true,
            },
            content: Content {
                request_line: RequestLine {
                    method: "GET".to_string(),
                    uri: "/".to_string(),
                    version: "HTTP/1.1".to_string(),
                },
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("host".to_string(), "$(environment.NONEXISTENT_HOST)".to_string());
                    headers
                },
                body: None,
            },
            footer: Footer::default(),
            file_path: "test-env.greq".to_string(),
        };

        let result = replace_placeholders_in_greq_file_with_optional_response(&mut greq_file, None);

        assert!(result.is_err());
        if let Err(error) = result {
            let error_message = error.to_string();
            assert!(error_message.contains("test-env.greq"));
            assert!(error_message.contains("header 'host'"));
            assert!(error_message.contains("Environment variable 'NONEXISTENT_HOST' not found"));
        }
    }

    #[test]
    fn test_dependency_failure_placeholder_replacement() {
        use std::collections::HashMap;
        use crate::models::{GreqFile, Header, Content, RequestLine, Footer};

        let mut greq_file = GreqFile {
            header: Header {
                project: Some("Test".to_string()),
                is_http: true,
                delimiter: "====".to_string(),
                extends: None,
                number_of_retries: 0,
                depends_on: Some("dependency-file".to_string()),
                timeout: None,
                allow_dependency_failure: true,
                show_warnings: true,
            },
            content: Content {
                request_line: RequestLine {
                    method: "GET".to_string(),
                    uri: "/".to_string(),
                    version: "HTTP/1.1".to_string(),
                },
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("authorization".to_string(), "Bearer $(dependency.status-code)".to_string());
                    headers.insert("host".to_string(), "$(environment.TEST_HOST)".to_string());
                    headers
                },
                body: None,
            },
            footer: Footer::default(),
            file_path: "test-file.greq".to_string(),
        };

        // Set environment variable for testing
        std::env::set_var("TEST_HOST", "example.com");

        let result = replace_placeholders_in_greq_file_with_dependency_handling(&mut greq_file, None, true);

        assert!(result.is_ok());
        // Dependency placeholder should be empty
        assert_eq!(greq_file.content.headers.get("authorization"), Some(&"Bearer ".to_string()));
        // Environment placeholder should still work
        assert_eq!(greq_file.content.headers.get("host"), Some(&"example.com".to_string()));

        // Clean up
        std::env::remove_var("TEST_HOST");
    }

    #[test]
    fn test_no_warning_when_show_warnings_false() {
        use std::collections::HashMap;
        use crate::models::{GreqFile, Header, Content, RequestLine, Footer};

        let mut greq_file = GreqFile {
            header: Header {
                project: Some("Test".to_string()),
                is_http: true,
                delimiter: "====".to_string(),
                extends: None,
                number_of_retries: 0,
                depends_on: Some("dependency-file".to_string()),
                timeout: None,
                allow_dependency_failure: true,
                show_warnings: false, // Warnings disabled
            },
            content: Content {
                request_line: RequestLine {
                    method: "GET".to_string(),
                    uri: "/".to_string(),
                    version: "HTTP/1.1".to_string(),
                },
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("authorization".to_string(), "Bearer $(dependency.status-code)".to_string());
                    headers
                },
                body: None,
            },
            footer: Footer::default(),
            file_path: "test-file.greq".to_string(),
        };

        let result = replace_placeholders_in_greq_file_with_dependency_handling(&mut greq_file, None, true);

        assert!(result.is_ok());
        // Dependency placeholder should still be replaced with empty string
        assert_eq!(greq_file.content.headers.get("authorization"), Some(&"Bearer ".to_string()));
        // No warning should be logged (we can't easily test log output, but function should complete without error)
    }

    #[test]
    fn test_dep_prefix_replacement() {
        let content = "Authorization: Bearer $(dep.response-body.json.token)";
        let result = replace_dependency_placeholders_with_empty_string(content, "test.greq", "header", false);
        assert_eq!(result.unwrap(), "Authorization: Bearer ");
    }

    #[test]
    fn test_mixed_dependency_prefixes() {
        let content = "$(dependency.status-code) and $(dep.response-body.json.id)";
        let result = replace_dependency_placeholders_with_empty_string(content, "test.greq", "header", false);
        assert_eq!(result.unwrap(), " and ");
    }

    #[test]
    fn test_dependency_placeholder_validation_without_depends_on() {
        use std::collections::HashMap;
        use crate::models::{GreqFile, Header, Content, RequestLine, Footer};

        let mut greq_file = GreqFile {
            header: Header {
                project: Some("Test".to_string()),
                is_http: true,
                delimiter: "====".to_string(),
                extends: None,
                depends_on: None, // No dependency defined
                allow_dependency_failure: false,
                show_warnings: true,
                timeout: None,
                number_of_retries: 0,
            },
            content: Content {
                request_line: RequestLine {
                    method: "GET".to_string(),
                    uri: "/test".to_string(),
                    version: "HTTP/1.1".to_string(),
                },
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("authorization".to_string(), "Bearer $(dependency.token)".to_string());
                    headers
                },
                body: None,
            },
            footer: Footer::default(),
            file_path: "test-file.greq".to_string(),
        };

        let result = replace_placeholders_in_greq_file_with_dependency_handling(&mut greq_file, None, false);
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Dependency placeholder 'dependency.token' found but no 'depends-on' is defined"));
    }

    #[test]
    fn test_dep_prefix_validation_without_depends_on() {
        use std::collections::HashMap;
        use crate::models::{GreqFile, Header, Content, RequestLine, Footer};

        let mut greq_file = GreqFile {
            header: Header {
                project: Some("Test".to_string()),
                is_http: true,
                delimiter: "====".to_string(),
                extends: None,
                depends_on: None, // No dependency defined
                allow_dependency_failure: false,
                show_warnings: true,
                timeout: None,
                number_of_retries: 0,
            },
            content: Content {
                request_line: RequestLine {
                    method: "POST".to_string(),
                    uri: "/users/$(dep.response-body.id)".to_string(),
                    version: "HTTP/1.1".to_string(),
                },
                headers: HashMap::new(),
                body: Some("{}".to_string()),
            },
            footer: Footer::default(),
            file_path: "test-file.greq".to_string(),
        };

        let result = replace_placeholders_in_greq_file_with_dependency_handling(&mut greq_file, None, false);
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Dependency placeholder 'dep.response-body.id' found but no 'depends-on' is defined"));
    }

    #[test]
    fn test_dependency_placeholder_validation_with_depends_on() {
        use std::collections::HashMap;
        use crate::models::{GreqFile, Header, Content, RequestLine, Footer};

        let mut greq_file = GreqFile {
            header: Header {
                project: Some("Test".to_string()),
                is_http: true,
                delimiter: "====".to_string(),
                extends: None,
                depends_on: Some("dependency.greq".to_string()), // Has dependency
                allow_dependency_failure: false,
                show_warnings: true,
                timeout: None,
                number_of_retries: 0,
            },
            content: Content {
                request_line: RequestLine {
                    method: "GET".to_string(),
                    uri: "/test".to_string(),
                    version: "HTTP/1.1".to_string(),
                },
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("authorization".to_string(), "Bearer $(dependency.response-body.token)".to_string());
                    headers
                },
                body: None,
            },
            footer: Footer::default(),
            file_path: "test-file.greq".to_string(),
        };

        // Create a dummy response
        let dummy_response = Response {
            status_code: 200,
            headers: HashMap::new(),
            body: r#"{"token": "abc123"}"#.to_string(),
            latency: std::time::Duration::from_millis(100),
        };

        let result = replace_placeholders_in_greq_file_with_dependency_handling(&mut greq_file, Some(&dummy_response), false);
        
        // Should succeed because depends_on is defined
        assert!(result.is_ok());
    }
}
