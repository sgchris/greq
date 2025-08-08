use crate::models::{Condition, ConditionKey, Operator, Response};
use crate::error::{GreqError, Result};
use regex::Regex;
use serde_json::Value;

/// Evaluate all conditions against a response
pub fn evaluate_conditions(conditions: &[Condition], response: &Response) -> Result<Vec<String>> {
    let mut failed_conditions = Vec::new();
    let condition_groups = group_conditions(conditions);
    
    for group in condition_groups {
        if !evaluate_condition_group(&group, response)? {
            let failed_desc = format_failed_conditions(&group);
            failed_conditions.push(failed_desc);
        }
    }
    
    Ok(failed_conditions)
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

/// Evaluate a group of conditions (connected by OR)
fn evaluate_condition_group(group: &[&Condition], response: &Response) -> Result<bool> {
    for condition in group {
        if evaluate_single_condition(condition, response)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Evaluate a single condition
fn evaluate_single_condition(condition: &Condition, response: &Response) -> Result<bool> {
    let actual_value = extract_condition_value(&condition.key, response)?;
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
        Operator::LessThan => compare_numeric(&actual_value, expected_value, |a, b| a < b)?,
        Operator::LessThanOrEqual => compare_numeric(&actual_value, expected_value, |a, b| a <= b)?,
        Operator::GreaterThan => compare_numeric(&actual_value, expected_value, |a, b| a > b)?,
        Operator::GreaterThanOrEqual => compare_numeric(&actual_value, expected_value, |a, b| a >= b)?,
        Operator::StartsWith => compare_starts_with(&actual_value, expected_value, condition.case_sensitive),
        Operator::EndsWith => compare_ends_with(&actual_value, expected_value, condition.case_sensitive),
        Operator::Exists => compare_exists(&actual_value, expected_value)?,
    };
    
    let final_result = if condition.is_not { !result } else { result };
    
    log::debug!(
        "Condition result: {final_result} (actual: '{actual_value}', expected: '{expected_value}')"
    );
    
    Ok(final_result)
}

/// Extract the actual value for a condition key from the response
fn extract_condition_value(key: &ConditionKey, response: &Response) -> Result<String> {
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
            extract_json_path_value(&response.body, path)
        },
    }
}

/// Extract value from JSON response body using path
fn extract_json_path_value(json_text: &str, path: &str) -> Result<String> {
    let value: Value = serde_json::from_str(json_text)
        .map_err(|_| GreqError::ConditionFailed("Response body is not valid JSON".to_string()))?;
    
    let result = navigate_json_path(&value, path)?;
    
    match result {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok("null".to_string()),
        _ => Ok(serde_json::to_string(&result)?),
    }
}

/// Navigate JSON path similar to placeholders module
fn navigate_json_path(value: &Value, path: &str) -> Result<Value> {
    let mut current = value;
    let parts = parse_json_path(path)?;
    
    for part in parts {
        match part {
            PathPart::Property(key) => {
                if let Value::Object(obj) = current {
                    current = obj.get(&key)
                        .ok_or_else(|| GreqError::ConditionFailed(format!("Property '{key}' not found")))?;
                } else {
                    return Err(GreqError::ConditionFailed(format!("Cannot access property '{key}' on non-object")));
                }
            },
            PathPart::Index(index) => {
                if let Value::Array(arr) = current {
                    current = arr.get(index)
                        .ok_or_else(|| GreqError::ConditionFailed(format!("Array index {index} out of bounds")))?;
                } else {
                    return Err(GreqError::ConditionFailed(format!("Cannot access index {index} on non-array")));
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
                
                let mut index_str = String::new();
                for ch in chars.by_ref() {
                    if ch == ']' {
                        break;
                    }
                    index_str.push(ch);
                }
                
                let index: usize = index_str.parse()
                    .map_err(|_| GreqError::ConditionFailed(format!("Invalid array index: {index_str}")))?;
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

fn compare_numeric<F>(actual: &str, expected: &str, op: F) -> Result<bool>
where
    F: Fn(f64, f64) -> bool,
{
    let actual_num: f64 = actual.parse()
        .map_err(|_| GreqError::ConditionFailed(format!("Cannot parse '{actual}' as number")))?;
    let expected_num: f64 = expected.parse()
        .map_err(|_| GreqError::ConditionFailed(format!("Cannot parse '{expected}' as number")))?;
    
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

fn compare_exists(actual: &str, expected: &str) -> Result<bool> {
    let expected_exists: bool = expected.parse()
        .map_err(|_| GreqError::ConditionFailed(format!("Invalid boolean value for exists: {expected}")))?;
    
    let actual_exists = !actual.is_empty();
    Ok(actual_exists == expected_exists)
}

// Formatting functions for error messages
fn format_failed_conditions(conditions: &[&Condition]) -> String {
    conditions.iter()
        .map(|c| format_condition(c))
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn format_condition(condition: &Condition) -> String {
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
        
        let result = evaluate_single_condition(&condition, &response).unwrap();
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
        
        let result = evaluate_single_condition(&condition, &response).unwrap();
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
        
        let result = evaluate_single_condition(&condition, &response).unwrap();
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
        
        let result = evaluate_single_condition(&condition, &response).unwrap();
        assert!(result); // NOT 404 should be true for 200
    }
}
