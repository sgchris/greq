use greq::greq_object::greq_footer_condition::*;
use greq::greq_object::greq_evaluator::*;
use greq::greq_object::greq_response::*;
use std::collections::HashMap;

fn create_test_response() -> GreqResponse {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("x-custom-header".to_string(), "custom-value".to_string());
    headers.insert("authorization".to_string(), "Bearer token123".to_string());
    headers.insert("content-length".to_string(), "1024".to_string());

    GreqResponse {
        status_code: 200,
        headers,
        body: Some("{\"message\": \"Hello World\"}".to_string()),
        ..Default::default()
    }
}

fn create_condition(
    key: &str,
    value: &str,
    operator: ConditionOperator,
    is_case_sensitive: bool,
    has_not: bool,
    is_comment: bool,
) -> GreqFooterCondition {
    GreqFooterCondition {
        original_line: format!("{} {} {} {} {}", key, value, "operator", is_case_sensitive, has_not),
        key: key.to_string(),
        value: value.to_string(),
        operator,
        is_case_sensitive,
        has_not,
        is_comment,
        ..Default::default()
    }
}

#[test]
fn test_comment_condition_always_returns_true() {
    let response = create_test_response();
    let condition = create_condition("any-key", "any-value", ConditionOperator::Equals, true, false, true);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_status_code_equals() {
    let response = create_test_response();
    let condition = create_condition("status-code", "200", ConditionOperator::Equals, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_status_code_not_equals() {
    let response = create_test_response();
    let condition = create_condition("status-code", "404", ConditionOperator::Equals, true, false, false);

    assert!(!GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_status_code_greater_than() {
    let response = create_test_response();
    let condition = create_condition("status-code", "199", ConditionOperator::GreaterThan, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_status_code_less_than() {
    let response = create_test_response();
    let condition = create_condition("status-code", "201", ConditionOperator::LessThan, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_response_body_equals() {
    let response = create_test_response();
    let condition = create_condition("response-body", "{\"message\": \"Hello World\"}", ConditionOperator::Equals, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_response_body_contains() {
    let response = create_test_response();
    let condition = create_condition("response-body", "Hello World", ConditionOperator::Contains, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_response_body_starts_with() {
    let response = create_test_response();
    let condition = create_condition("response-body", "{\"message\"", ConditionOperator::StartsWith, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_response_body_ends_with() {
    let response = create_test_response();
    let condition = create_condition("response-body", "World\"}", ConditionOperator::EndsWith, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_response_body_regex_match() {
    let response = create_test_response();
    let condition = create_condition("response-body", r#"\{"message":\s*"[^"]+"\}"#, ConditionOperator::MatchesRegex, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_response_body_regex_no_match() {
    let response = create_test_response();
    let condition = create_condition("response-body", r#"\{"error":\s*"[^"]+"\}"#, ConditionOperator::MatchesRegex, true, false, false);

    assert!(!GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_response_body_invalid_regex() {
    let response = create_test_response();
    let condition = create_condition("response-body", "[invalid regex", ConditionOperator::MatchesRegex, true, false, false);

    assert!(!GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_header_with_prefix_equals() {
    let response = create_test_response();
    let condition = create_condition("headers.content-type", "application/json", ConditionOperator::Equals, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_header_direct_access_equals() {
    let response = create_test_response();
    let condition = create_condition("content-type", "application/json", ConditionOperator::Equals, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_header_contains() {
    let response = create_test_response();
    let condition = create_condition("headers.authorization", "Bearer", ConditionOperator::Contains, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_header_not_exists() {
    let response = create_test_response();
    let condition = create_condition("headers.non-existent", "any-value", ConditionOperator::Equals, true, false, false);

    assert!(!GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_case_sensitive_comparison() {
    let response = create_test_response();
    let condition = create_condition("headers.content-type", "APPLICATION/JSON", ConditionOperator::Equals, true, false, false);

    assert!(!GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_case_insensitive_comparison() {
    let response = create_test_response();
    let condition = create_condition("headers.content-type", "APPLICATION/JSON", ConditionOperator::Equals, false, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_case_insensitive_contains() {
    let response = create_test_response();
    let condition = create_condition("response-body", "HELLO", ConditionOperator::Contains, false, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_has_not_flag_true() {
    let response = create_test_response();
    let condition = create_condition("status-code", "404", ConditionOperator::Equals, true, true, false);

    assert!(GreqEvaluator::evaluate(&response, &condition)); // NOT equals 404 = true
}

#[test]
fn test_has_not_flag_false() {
    let response = create_test_response();
    let condition = create_condition("status-code", "200", ConditionOperator::Equals, true, true, false);

    assert!(!GreqEvaluator::evaluate(&response, &condition)); // NOT equals 200 = false
}

#[test]
fn test_numeric_header_greater_than() {
    let response = create_test_response();
    let condition = create_condition("headers.content-length", "1000", ConditionOperator::GreaterThan, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_numeric_header_less_than() {
    let response = create_test_response();
    let condition = create_condition("headers.content-length", "2000", ConditionOperator::LessThan, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_non_numeric_greater_than_returns_false() {
    let response = create_test_response();
    let condition = create_condition("headers.content-type", "application", ConditionOperator::GreaterThan, true, false, false);

    assert!(!GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_empty_response_body() {
    let mut response = create_test_response();
    response.body = None;
    let condition = create_condition("response-body", "", ConditionOperator::Equals, true, false, false);

    assert!(GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_empty_response_body_with_non_empty_expected() {
    let mut response = create_test_response();
    response.body = None;
    let condition = create_condition("response-body", "something", ConditionOperator::Equals, true, false, false);

    assert!(!GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_complex_case_insensitive_with_not() {
    let response = create_test_response();
    let condition = create_condition("headers.content-type", "TEXT/HTML", ConditionOperator::Equals, false, true, false);

    assert!(GreqEvaluator::evaluate(&response, &condition)); // NOT equals text/html = true
}

#[test]
fn test_regex_case_sensitivity() {
    let response = create_test_response();

    // Case sensitive regex should not match
    let condition_sensitive = create_condition("response-body", "HELLO", ConditionOperator::MatchesRegex, true, false, false);
    assert!(!GreqEvaluator::evaluate(&response, &condition_sensitive));

    // Case insensitive regex should match (lowercase conversion happens before regex)
    let condition_insensitive = create_condition("response-body", "HELLO", ConditionOperator::MatchesRegex, false, false, false);
    assert!(GreqEvaluator::evaluate(&response, &condition_insensitive));
}

#[test]
fn test_edge_case_empty_key() {
    let response = create_test_response();
    let condition = create_condition("", "any-value", ConditionOperator::Equals, true, false, false);

    // Should look for header with empty name, which doesn't exist
    assert!(!GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_edge_case_malformed_headers_prefix() {
    let response = create_test_response();
    let condition = create_condition("headers.", "any-value", ConditionOperator::Equals, true, false, false);

    // Should return empty string for malformed header key
    assert!(!GreqEvaluator::evaluate(&response, &condition));
}

#[test]
fn test_floating_point_comparison() {
    let mut response = create_test_response();
    response.headers.insert("rate-limit".to_string(), "99.5".to_string());

    let condition = create_condition("headers.rate-limit", "99.0", ConditionOperator::GreaterThan, true, false, false);
    assert!(GreqEvaluator::evaluate(&response, &condition));

    let condition2 = create_condition("headers.rate-limit", "100.0", ConditionOperator::LessThan, true, false, false);
    assert!(GreqEvaluator::evaluate(&response, &condition2));
}
