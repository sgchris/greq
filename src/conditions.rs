use crate::models::{Condition, ConditionKey, Operator, Response};
use crate::error::{GreqError, Result};
use regex::Regex;
use serde_json::Value;

/// Evaluate all conditions against a response
/// Evaluate conditions against a response with file context for better error reporting
/// Stops at the first failing condition to preserve evaluation order
pub fn evaluate_conditions(conditions: &[Condition], response: &Response, file_path: &str) -> Result<Vec<String>> {
    let condition_groups = group_conditions(conditions);
    
    for group in condition_groups {
        if let Some(failed_desc) = evaluate_condition_group_with_details(&group, response, file_path)? {
            // Return immediately on first failure to preserve order
            return Ok(vec![failed_desc]);
        }
    }
    
    Ok(Vec::new())
}

/// Group conditions by OR relationships
fn group_conditions(conditions: &[Condition]) -> Vec<Vec<&Condition>> {
    let mut groups = Vec::new();
    let mut current_group = Vec::new();
    
    for condition in conditions {
        if condition.is_or && !current_group.is_empty() {
            current_group.push(condition);
        } else {
            if !current_group.is_empty() {
                groups.push(current_group);
            }
            current_group = vec![condition];
        }
    }
    
    if !current_group.is_empty() {
        groups.push(current_group);
    }
    
    groups
}

/// Evaluate a group of conditions (connected by OR) and return failure details if any
/// Shows error from the first condition in the group if all conditions fail
fn evaluate_condition_group_with_details(group: &[&Condition], response: &Response, file_path: &str) -> Result<Option<String>> {
    let mut first_failure: Option<String> = None;
    
    for condition in group {
        match evaluate_single_condition_with_details(condition, response, file_path)? {
            ConditionResult::Passed => return Ok(None), // If any condition passes in OR group, group passes
            ConditionResult::Failed { actual_value, condition } => {
                // Store only the first failure
                if first_failure.is_none() {
                    first_failure = Some(format_failed_condition_with_actual(&condition, &actual_value));
                }
            }
        }
    }
    
    // All conditions failed, return the first failure
    Ok(first_failure)
}

#[derive(Debug)]
enum ConditionResult {
    Passed,
    Failed { actual_value: String, condition: Condition },
}

/// Evaluate a single condition with detailed results
fn evaluate_single_condition_with_details(condition: &Condition, response: &Response, file_path: &str) -> Result<ConditionResult> {
    // For exists operator, we need to handle missing JSON paths gracefully
    let actual_value = if condition.operator == Operator::Exists {
        extract_condition_value_for_exists(&condition.key, response, file_path)
    } else {
        extract_condition_value(&condition.key, response, file_path)?
    };
    
    let expected_value = &condition.value;
    
    log::debug!(
        "Evaluating condition: {} {} {} (case_sensitive: {})",
        format_condition_key(&condition.key),
        format_operator(&condition.operator),
        expected_value,
        condition.case_sensitive
    );
    
    let result = match condition.operator {
        Operator::Equals => compare_equals(&actual_value, expected_value, condition.case_sensitive),
        Operator::Contains => compare_contains(&actual_value, expected_value, condition.case_sensitive),
        Operator::MatchesRegex => compare_regex(&actual_value, expected_value)?,
        Operator::LessThan => compare_numeric(&actual_value, expected_value, file_path, |a, b| a < b)?,
        Operator::LessThanOrEqual => compare_numeric(&actual_value, expected_value, file_path, |a, b| a <= b)?,
        Operator::GreaterThan => compare_numeric(&actual_value, expected_value, file_path, |a, b| a > b)?,
        Operator::GreaterThanOrEqual => compare_numeric(&actual_value, expected_value, file_path, |a, b| a >= b)?,
        Operator::StartsWith => compare_starts_with(&actual_value, expected_value, condition.case_sensitive),
        Operator::EndsWith => compare_ends_with(&actual_value, expected_value, condition.case_sensitive),
        Operator::Exists => compare_exists(&actual_value, expected_value, file_path)?,
    };
    
    let final_result = if condition.is_not { !result } else { result };
    
    log::debug!(
        "Condition result: {final_result} (actual: '{actual_value}', expected: '{expected_value}')"
    );
    
    if final_result {
        Ok(ConditionResult::Passed)
    } else {
        Ok(ConditionResult::Failed { 
            actual_value, 
            condition: condition.clone(),
        })
    }
}

/// Extract the actual value for a condition key from the response
fn extract_condition_value(key: &ConditionKey, response: &Response, file_path: &str) -> Result<String> {
    match key {
        ConditionKey::StatusCode => Ok(response.status_code.to_string()),
        ConditionKey::Latency => Ok(response.latency.as_millis().to_string()),
        ConditionKey::ResponseBody => Ok(response.body.clone()),
        ConditionKey::Headers => {
            // Return all headers as a formatted string for contains checks
            let headers_str = response.headers.iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(headers_str)
        },
        ConditionKey::Header(header_name) => {
            Ok(response.headers.get(&header_name.to_lowercase()).cloned().unwrap_or_default())
        },
        ConditionKey::ResponseBodyPath(path) => {
            extract_json_path_value(&response.body, path, file_path)
        },
    }
}

/// Extract value for exists operator - returns empty string if path doesn't exist
/// This allows the exists operator to properly check for non-existence
fn extract_condition_value_for_exists(key: &ConditionKey, response: &Response, file_path: &str) -> String {
    match key {
        ConditionKey::StatusCode => response.status_code.to_string(),
        ConditionKey::Latency => response.latency.as_millis().to_string(),
        ConditionKey::ResponseBody => response.body.clone(),
        ConditionKey::Headers => {
            let headers_str = response.headers.iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>()
                .join("\n");
            headers_str
        },
        ConditionKey::Header(header_name) => {
            response.headers.get(&header_name.to_lowercase()).cloned().unwrap_or_default()
        },
        ConditionKey::ResponseBodyPath(path) => {
            // For exists operator, return empty string if path doesn't exist
            extract_json_path_value(&response.body, path, file_path).unwrap_or_default()
        },
    }
}

/// Extract value from JSON response body using path
fn extract_json_path_value(json_text: &str, path: &str, file_path: &str) -> Result<String> {
    let value: Value = serde_json::from_str(json_text)
        .map_err(|_| GreqError::ConditionFailed(format!("{}: Response body is not valid JSON", file_path)))?;
    
    let result = navigate_json_path(&value, path, file_path)?;
    
    match result {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok("null".to_string()),
        _ => Ok(serde_json::to_string(&result)?),
    }
}

/// Navigate JSON path similar to placeholders module
fn navigate_json_path(value: &Value, path: &str, file_path: &str) -> Result<Value> {
    let mut current = value;
    let parts = parse_json_path(path, file_path)?;
    
    for part in parts {
        match part {
            PathPart::Property(key) => {
                if let Value::Object(obj) = current {
                    current = obj.get(&key)
                        .ok_or_else(|| GreqError::ConditionFailed(format!("{}: Property '{}' not found in JSON path '{}'", file_path, key, path)))?;
                } else {
                    return Err(GreqError::ConditionFailed(format!("{}: Cannot access property '{}' on non-object in JSON path '{}'", file_path, key, path)));
                }
            },
            PathPart::Index(index) => {
                if let Value::Array(arr) = current {
                    current = arr.get(index)
                        .ok_or_else(|| GreqError::ConditionFailed(format!("{}: Array index {} out of bounds in JSON path '{}'", file_path, index, path)))?;
                } else {
                    return Err(GreqError::ConditionFailed(format!("{}: Cannot access index {} on non-array in JSON path '{}'", file_path, index, path)));
                }
            },
        }
    }
    
    Ok(current.clone())
}

#[derive(Debug, Clone)]
enum PathPart {
    Property(String),
    Index(usize),
}

fn parse_json_path(path: &str, file_path: &str) -> Result<Vec<PathPart>> {
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
                
                let mut index_str = String::new();
                for ch in chars.by_ref() {
                    if ch == ']' {
                        break;
                    }
                    index_str.push(ch);
                }
                
                let index: usize = index_str.parse()
                    .map_err(|_| GreqError::ConditionFailed(format!("{}: Invalid array index: {index_str}", file_path)))?;
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

// Comparison functions
fn compare_equals(actual: &str, expected: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        actual == expected
    } else {
        actual.to_lowercase() == expected.to_lowercase()
    }
}

fn compare_contains(actual: &str, expected: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        actual.contains(expected)
    } else {
        actual.to_lowercase().contains(&expected.to_lowercase())
    }
}

fn compare_regex(actual: &str, pattern: &str) -> Result<bool> {
    let regex = Regex::new(pattern)?;
    Ok(regex.is_match(actual))
}

fn compare_numeric<F>(actual: &str, expected: &str, file_path: &str, op: F) -> Result<bool>
where
    F: Fn(f64, f64) -> bool,
{
    let actual_num: f64 = actual.parse()
        .map_err(|_| GreqError::ConditionFailed(format!("{}: Cannot parse '{actual}' as number", file_path)))?;
    let expected_num: f64 = expected.parse()
        .map_err(|_| GreqError::ConditionFailed(format!("{}: Cannot parse '{expected}' as number", file_path)))?;
    
    Ok(op(actual_num, expected_num))
}

fn compare_starts_with(actual: &str, expected: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        actual.starts_with(expected)
    } else {
        actual.to_lowercase().starts_with(&expected.to_lowercase())
    }
}

fn compare_ends_with(actual: &str, expected: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        actual.ends_with(expected)
    } else {
        actual.to_lowercase().ends_with(&expected.to_lowercase())
    }
}

fn compare_exists(actual: &str, expected: &str, file_path: &str) -> Result<bool> {
    let expected_exists: bool = expected.parse()
        .map_err(|_| GreqError::ConditionFailed(format!("{}: Invalid boolean value for exists: {expected}", file_path)))?;
    
    let actual_exists = !actual.is_empty();
    Ok(actual_exists == expected_exists)
}

// Formatting functions for error messages
fn format_failed_condition_with_actual(condition: &Condition, actual_value: &str) -> String {
    let mut parts = Vec::new();
    
    if condition.is_not {
        parts.push("NOT".to_string());
    }
    
    parts.push(format_condition_key(&condition.key));
    parts.push(format_operator(&condition.operator));
    
    if condition.case_sensitive {
        parts.push("case-sensitive".to_string());
    }
    
    parts.push(format!("'{}'", condition.value));
    parts.push(format!("(actual: '{}')", actual_value));
    
    parts.join(" ")
}

fn format_condition_key(key: &ConditionKey) -> String {
    match key {
        ConditionKey::StatusCode => "status-code".to_string(),
        ConditionKey::Latency => "latency".to_string(),
        ConditionKey::ResponseBody => "response-body".to_string(),
        ConditionKey::Headers => "headers".to_string(),
        ConditionKey::Header(name) => format!("headers.{name}"),
        ConditionKey::ResponseBodyPath(path) => format!("response-body.{path}"),
    }
}

fn format_operator(operator: &Operator) -> String {
    match operator {
        Operator::Equals => "equals".to_string(),
        Operator::Contains => "contains".to_string(),
        Operator::MatchesRegex => "matches-regex".to_string(),
        Operator::LessThan => "less-than".to_string(),
        Operator::LessThanOrEqual => "less-than-or-equal".to_string(),
        Operator::GreaterThan => "greater-than".to_string(),
        Operator::GreaterThanOrEqual => "greater-than-or-equal".to_string(),
        Operator::StartsWith => "starts-with".to_string(),
        Operator::EndsWith => "ends-with".to_string(),
        Operator::Exists => "exists".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;
    
    // Helper function for tests to get just the boolean result
    fn evaluate_single_condition_test(condition: &Condition, response: &Response) -> Result<bool> {
        match evaluate_single_condition_with_details(condition, response, "test-file.greq")? {
            ConditionResult::Passed => Ok(true),
            ConditionResult::Failed { .. } => Ok(false),
        }
    }
    
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
    fn test_evaluate_status_code_condition() {
        let response = create_test_response();
        let condition = Condition {
            is_or: false,
            is_not: false,
            key: ConditionKey::StatusCode,
            operator: Operator::Equals,
            case_sensitive: false,
            value: "200".to_string(),
        };
        
        let result = evaluate_single_condition_test(&condition, &response).unwrap();
        assert!(result);
    }
    
    #[test]
    fn test_evaluate_response_body_contains() {
        let response = create_test_response();
        let condition = Condition {
            is_or: false,
            is_not: false,
            key: ConditionKey::ResponseBody,
            operator: Operator::Contains,
            case_sensitive: false,
            value: "test".to_string(),
        };
        
        let result = evaluate_single_condition_test(&condition, &response).unwrap();
        assert!(result);
    }
    
    #[test]
    fn test_evaluate_json_path_condition() {
        let response = create_test_response();
        let condition = Condition {
            is_or: false,
            is_not: false,
            key: ConditionKey::ResponseBodyPath("id".to_string()),
            operator: Operator::Equals,
            case_sensitive: false,
            value: "123".to_string(),
        };
        
        let result = evaluate_single_condition_test(&condition, &response).unwrap();
        assert!(result);
    }
    
    #[test]
    fn test_evaluate_not_condition() {
        let response = create_test_response();
        let condition = Condition {
            is_or: false,
            is_not: true,
            key: ConditionKey::StatusCode,
            operator: Operator::Equals,
            case_sensitive: false,
            value: "404".to_string(),
        };
        
        let result = evaluate_single_condition_test(&condition, &response).unwrap();
        assert!(result); // NOT 404 should be true for 200
    }
    
    #[test]
    fn test_exists_operator_with_existing_field() {
        let response = create_test_response();
        let condition = Condition {
            is_or: false,
            is_not: false,
            key: ConditionKey::ResponseBodyPath("id".to_string()),
            operator: Operator::Exists,
            case_sensitive: false,
            value: "true".to_string(),
        };
        
        let result = evaluate_single_condition_test(&condition, &response).unwrap();
        assert!(result); // id exists in the response
    }
    
    #[test]
    fn test_exists_operator_with_missing_field() {
        let response = create_test_response();
        let condition = Condition {
            is_or: false,
            is_not: false,
            key: ConditionKey::ResponseBodyPath("nonexistent".to_string()),
            operator: Operator::Exists,
            case_sensitive: false,
            value: "false".to_string(),
        };
        
        let result = evaluate_single_condition_test(&condition, &response).unwrap();
        assert!(result); // nonexistent field should have exists: false
    }
    
    #[test]
    fn test_not_exists_operator_with_missing_field() {
        let response = create_test_response();
        // This is the bug fix test: "not response-body.data exists: true"
        // When data field doesn't exist, this should PASS
        let condition = Condition {
            is_or: false,
            is_not: true,
            key: ConditionKey::ResponseBodyPath("data".to_string()),
            operator: Operator::Exists,
            case_sensitive: false,
            value: "true".to_string(),
        };
        
        let result = evaluate_single_condition_test(&condition, &response).unwrap();
        assert!(result); // NOT exists:true should be true when field is missing
    }
    
    #[test]
    fn test_not_exists_operator_with_existing_field() {
        let response = create_test_response();
        // When field exists, "not exists: true" should FAIL
        let condition = Condition {
            is_or: false,
            is_not: true,
            key: ConditionKey::ResponseBodyPath("id".to_string()),
            operator: Operator::Exists,
            case_sensitive: false,
            value: "true".to_string(),
        };
        
        let result = evaluate_single_condition_test(&condition, &response).unwrap();
        assert!(!result); // NOT exists:true should be false when field exists
    }
}
