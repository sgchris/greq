use greq::greq_object::greq_footer::{GreqFooter, GreqFooterError};
use greq::greq_object::greq_footer_condition::{ConditionOperator, GreqFooterCondition};
use greq::greq_object::greq_evaluator::GreqEvaluator;
use greq::greq_object::greq_response::GreqResponse;
use std::collections::HashMap;

// ============= GreqFooter Tests =============

#[test]
fn test_greq_footer_parse_comprehensive() {
    // Test parsing various valid conditions including all operators, prefixes, and edge cases
    let footer_lines = vec![
        "",  // empty line should be ignored
        "status-code equals: 200",
        "or not response-body contains case-sensitive: Error",
        "   headers.content-type   starts-with  :   application/json   ",  // whitespace handling
        "headers.x-api-key matches-regex: ^[a-zA-Z0-9]+$",
        "status-code greater-than: 199",
        "response-body ends-with: success"
    ];
    
    let result = GreqFooter::parse(&footer_lines, None, None);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 6);
    
    // Verify first condition
    let c1 = &footer.conditions[0];
    assert_eq!(c1.key, "status-code");
    assert_eq!(c1.operator, ConditionOperator::Equals);
    assert_eq!(c1.value, "200");
    assert!(!c1.has_or && !c1.has_not && !c1.is_case_sensitive);
    
    // Verify second condition (with or and not)
    let c2 = &footer.conditions[1];
    assert_eq!(c2.key, "response-body");
    assert_eq!(c2.operator, ConditionOperator::Contains);
    assert_eq!(c2.value, "Error");
    assert!(c2.has_or && c2.has_not && c2.is_case_sensitive);
    
    // Verify header condition with whitespace
    let c3 = &footer.conditions[2];
    assert_eq!(c3.key, "content-type");
    assert_eq!(c3.operator, ConditionOperator::StartsWith);
    assert_eq!(c3.value, "application/json");
    
    // Verify all operators are represented
    let operators: Vec<_> = footer.conditions.iter().map(|c| &c.operator).collect();
    assert!(operators.contains(&&ConditionOperator::Equals));
    assert!(operators.contains(&&ConditionOperator::Contains));
    assert!(operators.contains(&&ConditionOperator::StartsWith));
    assert!(operators.contains(&&ConditionOperator::MatchesRegex));
    assert!(operators.contains(&&ConditionOperator::GreaterThan));
    assert!(operators.contains(&&ConditionOperator::EndsWith));
}

#[test]
fn test_greq_footer_parse_all_error_cases() {
    let test_cases = vec![
        (vec!["status-code equals 200"], GreqFooterError::LineHasNoColonSign),
        (vec!["status-code equals:"], GreqFooterError::LineHasNoColonSign),
        (vec![": 200"], GreqFooterError::LineHasNoColonSign),
        (vec!["not or status-code equals: 200"], GreqFooterError::TheKeywordOrNotInTheBeginning),
        (vec!["not not status-code equals: 200"], GreqFooterError::TheKeywordNotAppearsMoreThanOnce),
        (vec!["headers. equals: value"], GreqFooterError::InvalidHeaderKey { header_key: "headers.".to_string() }),
        (vec!["unknown-keyword equals: value"], GreqFooterError::InvalidKey { key: "unknown-keyword".to_string() }),
        (vec!["status-code unknown-operator: 200"], GreqFooterError::InvalidKey { key: "unknown-operator".to_string() }),
    ];
    
    for (lines, expected_error) in test_cases {
        let result = GreqFooter::parse(&lines, None, None);
        assert!(result.is_err());
        let error = result.unwrap_err();
        match (&error, &expected_error) {
            (GreqFooterError::LineHasNoColonSign, GreqFooterError::LineHasNoColonSign) => {},
            (GreqFooterError::TheKeywordOrNotInTheBeginning, GreqFooterError::TheKeywordOrNotInTheBeginning) => {},
            (GreqFooterError::TheKeywordNotAppearsMoreThanOnce, GreqFooterError::TheKeywordNotAppearsMoreThanOnce) => {},
            (GreqFooterError::InvalidHeaderKey { header_key: actual }, GreqFooterError::InvalidHeaderKey { header_key: expected }) => {
                assert_eq!(actual, expected);
            },
            (GreqFooterError::InvalidKey { key: actual }, GreqFooterError::InvalidKey { key: expected }) => {
                assert_eq!(actual, expected);
            },
            _ => panic!("Expected {:?}, got {:?}", expected_error, error),
        }
    }
}

#[test]
fn test_greq_footer_enrich_with_comprehensive() {
    // Test the enrich_with functionality thoroughly
    
    // Base footer with some conditions
    let base_lines = vec![
        "status-code equals: 200",
        "headers.content-type contains: json"
    ];
    let base_footer = GreqFooter::parse(&base_lines, None, None).unwrap();
    
    // Case 1: Empty footer should inherit all from base
    let mut empty_footer = GreqFooter::default();
    empty_footer.enrich_with(&base_footer).unwrap();
    assert_eq!(empty_footer.conditions.len(), 2);
    assert_eq!(empty_footer.conditions[0].key, "status-code");
    assert_eq!(empty_footer.conditions[1].key, "content-type");
    
    // Case 2: Footer with existing conditions should merge non-duplicates
    let current_lines = vec![
        "response-body contains: success",
        "status-code equals: 200"  // duplicate from base
    ];
    let mut current_footer = GreqFooter::parse(&current_lines, None, None).unwrap();
    assert_eq!(current_footer.conditions.len(), 2);
    
    current_footer.enrich_with(&base_footer).unwrap();
    assert_eq!(current_footer.conditions.len(), 3); // Added only the non-duplicate
    
    // Verify the non-duplicate was added
    let has_content_type = current_footer.conditions.iter()
        .any(|c| c.key == "content-type");
    assert!(has_content_type);
    
    // Case 3: Test with complex conditions to ensure equals() method works properly
    let complex_base_lines = vec![
        "or not response-body contains case-sensitive: Error"
    ];
    let complex_base = GreqFooter::parse(&complex_base_lines, None, None).unwrap();
    
    let simple_lines = vec![
        "response-body contains: error"  // Different case sensitivity
    ];
    let mut simple_footer = GreqFooter::parse(&simple_lines, None, None).unwrap();
    
    simple_footer.enrich_with(&complex_base).unwrap();
    assert_eq!(simple_footer.conditions.len(), 2); // Both should exist as they're different
}

// ============= GreqEvaluator Tests =============

#[test]
fn test_greq_evaluator_all_operators() {
    // Create a mock response
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("x-api-version".to_string(), "v2.1.0".to_string());
    
    let response = GreqResponse {
        status_code: 201,
        body: Some("User created successfully with ID 12345".to_string()),
        headers,
        ..Default::default()
    };
    
    // Test all operators with various conditions
    let test_cases = vec![
        // Status code tests
        ("status-code", "201", ConditionOperator::Equals, false, false, true),
        ("status-code", "200", ConditionOperator::Equals, false, false, false),
        ("status-code", "200", ConditionOperator::GreaterThan, false, false, true),
        ("status-code", "300", ConditionOperator::LessThan, false, false, true),
        
        // Response body tests
        ("response-body", "User created", ConditionOperator::Contains, false, false, true),
        ("response-body", "user created", ConditionOperator::Contains, true, false, false), // case sensitive
        ("response-body", "user created", ConditionOperator::Contains, false, false, true), // case insensitive
        ("response-body", "User created", ConditionOperator::StartsWith, false, false, true),
        ("response-body", "12345", ConditionOperator::EndsWith, false, false, true),
        ("response-body", r"\d+", ConditionOperator::MatchesRegex, false, false, true),
        
        // Header tests
        ("content-type", "application/json", ConditionOperator::Equals, false, false, true),
        ("content-type", "application", ConditionOperator::StartsWith, false, false, true),
        ("x-api-version", "v2", ConditionOperator::StartsWith, false, false, true),
        
        // Negation tests
        ("status-code", "404", ConditionOperator::Equals, false, true, true), // not equals 404
        ("response-body", "error", ConditionOperator::Contains, false, true, true), // not contains error
    ];
    
    for (key, value, operator, case_sensitive, has_not, expected) in test_cases {
        let condition = GreqFooterCondition {
            key: key.to_string(),
            value: value.to_string(),
            operator,
            is_case_sensitive: case_sensitive,
            has_not,
            ..Default::default()
        };
        
        let result = GreqEvaluator::evaluate(&response, &condition);
        assert_eq!(result, expected, 
            "Failed for key: {}, value: {}, operator: {:?}, case_sensitive: {}, has_not: {}", 
            key, value, operator, case_sensitive, has_not
        );
    }
}

#[test]
fn test_greq_evaluator_edge_cases() {
    // Test edge cases and error scenarios
    let response = GreqResponse {
        status_code: 200,
        body: Some("".to_string()), // empty body
        headers: HashMap::new(), // no headers
        ..Default::default()
    };
    
    // Test with missing header
    let condition = GreqFooterCondition {
        key: "missing-header".to_string(),
        value: "any-value".to_string(),
        operator: ConditionOperator::Equals,
        ..Default::default()
    };
    assert!(!GreqEvaluator::evaluate(&response, &condition));
    
    // Test with empty body
    let condition = GreqFooterCondition {
        key: "response-body".to_string(),
        value: "".to_string(),
        operator: ConditionOperator::Equals,
        ..Default::default()
    };
    assert!(GreqEvaluator::evaluate(&response, &condition));
    
    // Test invalid regex
    let condition = GreqFooterCondition {
        key: "response-body".to_string(),
        value: "[invalid regex".to_string(),
        operator: ConditionOperator::MatchesRegex,
        ..Default::default()
    };
    assert!(!GreqEvaluator::evaluate(&response, &condition));
    
    // Test numeric comparison with non-numeric values
    let response_with_text = GreqResponse {
        status_code: 200,
        body: Some("not-a-number".to_string()),
        ..Default::default()
    };
    
    let condition = GreqFooterCondition {
        key: "response-body".to_string(),
        value: "100".to_string(),
        operator: ConditionOperator::GreaterThan,
        ..Default::default()
    };
    assert!(!GreqEvaluator::evaluate(&response_with_text, &condition));
}

#[test]
fn test_greq_evaluator_comment_and_complex_scenarios() {
    let response = GreqResponse {
        status_code: 500,
        body: Some("Internal Server Error".to_string()),
        ..Default::default()
    };
    
    // Test comment condition (should always return true)
    let comment_condition = GreqFooterCondition {
        is_comment: true,
        key: "any-key".to_string(),
        value: "any-value".to_string(),
        operator: ConditionOperator::Equals,
        ..Default::default()
    };
    assert!(GreqEvaluator::evaluate(&response, &comment_condition));
    
    // Test complex condition with multiple flags
    let complex_condition = GreqFooterCondition {
        key: "response-body".to_string(),
        value: "SUCCESS".to_string(),
        operator: ConditionOperator::Contains,
        is_case_sensitive: true,
        has_not: true, // not contains (case sensitive)
        has_or: true, // this doesn't affect evaluation, just parsing
        ..Default::default()
    };
    // Should return true because body doesn't contain "SUCCESS" (case sensitive)
    assert!(GreqEvaluator::evaluate(&response, &complex_condition));
    
    // Test the same but case insensitive
    let complex_condition_ci = GreqFooterCondition {
        key: "response-body".to_string(),
        value: "ERROR".to_string(),
        operator: ConditionOperator::Contains,
        is_case_sensitive: false,
        has_not: true, // not contains (case insensitive)
        ..Default::default()
    };
    // Should return false because body contains "error" when case insensitive
    assert!(!GreqEvaluator::evaluate(&response, &complex_condition_ci));
}
