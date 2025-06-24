use greq::greq_object::greq_footer::{GreqFooter, GreqFooterError};
use greq::greq_object::greq_footer_condition::{ConditionOperator};

#[test]
fn test_parse_empty_footer() {
    let footer_lines = vec![];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert!(footer.conditions.is_empty());
}

#[test]
fn test_parse_footer_with_empty_lines() {
    let footer_lines = vec!["", "   ", "\t"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert!(footer.conditions.is_empty());
}

#[test]
fn test_parse_basic_status_code_condition() {
    let footer_lines = vec!["status-code equals: 200"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    let condition = &footer.conditions[0];
    assert_eq!(condition.key, "status-code");
    assert_eq!(condition.operator, ConditionOperator::Equals);
    assert_eq!(condition.value, "200");
    assert!(!condition.has_or);
    assert!(!condition.has_not);
    assert!(!condition.is_case_sensitive);
}

#[test]
fn test_parse_response_body_with_contains() {
    let footer_lines = vec!["response-body contains: success"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    let condition = &footer.conditions[0];
    assert_eq!(condition.key, "response-body");
    assert_eq!(condition.operator, ConditionOperator::Contains);
    assert_eq!(condition.value, "success");
}

#[test]
fn test_parse_with_or_prefix() {
    let footer_lines = vec!["or status-code equals: 404"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    let condition = &footer.conditions[0];
    assert!(condition.has_or);
    assert_eq!(condition.key, "status-code");
    assert_eq!(condition.operator, ConditionOperator::Equals);
    assert_eq!(condition.value, "404");
}

#[test]
fn test_parse_with_not_prefix() {
    let footer_lines = vec!["not status-code equals: 500"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    let condition = &footer.conditions[0];
    assert!(condition.has_not);
    assert_eq!(condition.key, "status-code");
    assert_eq!(condition.operator, ConditionOperator::Equals);
    assert_eq!(condition.value, "500");
}

#[test]
fn test_parse_with_or_and_not_prefix() {
    let footer_lines = vec!["or not response-body contains: error"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    let condition = &footer.conditions[0];
    assert!(condition.has_or);
    assert!(condition.has_not);
    assert_eq!(condition.key, "response-body");
    assert_eq!(condition.operator, ConditionOperator::Contains);
    assert_eq!(condition.value, "error");
}

#[test]
fn test_parse_with_case_sensitive_suffix() {
    let footer_lines = vec!["response-body contains case-sensitive: Success"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    let condition = &footer.conditions[0];
    assert_eq!(condition.key, "response-body");
    assert_eq!(condition.operator, ConditionOperator::Contains);
    assert_eq!(condition.value, "Success");
    assert!(condition.is_case_sensitive);
}

#[test]
fn test_parse_all_operators() {
    let test_cases = vec![
        ("response-body equals: test", ConditionOperator::Equals),
        ("response-body contains: test", ConditionOperator::Contains),
        ("response-body starts-with: test", ConditionOperator::StartsWith),
        ("response-body ends-with: test", ConditionOperator::EndsWith),
        ("response-body matches-regex: test", ConditionOperator::MatchesRegex),
        ("status-code greater-than: 200", ConditionOperator::GreaterThan),
        ("status-code less-than: 500", ConditionOperator::LessThan),
    ];

    for (line, expected_operator) in test_cases {
        let footer_lines = vec![line];
        let result = GreqFooter::parse(&footer_lines);
        assert!(result.is_ok(), "Failed to parse: {}", line);
        let footer = result.unwrap();
        assert_eq!(footer.conditions.len(), 1);
        assert_eq!(footer.conditions[0].operator, expected_operator);
    }
}

#[test]
fn test_parse_headers_condition() {
    let footer_lines = vec!["headers.content-type equals: application/json"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    let condition = &footer.conditions[0];
    assert_eq!(condition.key, "content-type");
    assert_eq!(condition.operator, ConditionOperator::Equals);
    assert_eq!(condition.value, "application/json");
}

#[test]
fn test_parse_multiple_conditions() {
    let footer_lines = vec![
        "status-code equals: 200",
        "response-body contains: success",
        "headers.content-type equals: application/json"
    ];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 3);
}

#[test]
fn test_parse_complex_condition() {
    let footer_lines = vec!["or not response-body contains case-sensitive: Error Message"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    let condition = &footer.conditions[0];
    assert!(condition.has_or);
    assert!(condition.has_not);
    assert_eq!(condition.key, "response-body");
    assert_eq!(condition.operator, ConditionOperator::Contains);
    assert_eq!(condition.value, "Error Message");
    assert!(condition.is_case_sensitive);
}

#[test]
fn test_parse_value_with_special_characters() {
    let footer_lines = vec!["response-body equals: {\"status\": \"ok\", \"data\": [1,2,3]}"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    assert_eq!(footer.conditions[0].value, "{\"status\": \"ok\", \"data\": [1,2,3]}");
}

#[test]
fn test_parse_value_with_multiple_colons() {
    let footer_lines = vec!["response-body equals: http://example.com:8080/path"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    assert_eq!(footer.conditions[0].value, "http://example.com:8080/path");
}

// Error cases
#[test]
fn test_parse_line_without_colon() {
    let footer_lines = vec!["status-code equals 200"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_err());
    match result.unwrap_err() {
        GreqFooterError::LineHasNoColonSign => {},
        _ => panic!("Expected LineHasNoColonSign error"),
    }
}

#[test]
fn test_parse_line_with_empty_key() {
    let footer_lines = vec![": 200"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_err());
    match result.unwrap_err() {
        GreqFooterError::LineHasNoColonSign => {},
        _ => panic!("Expected LineHasNoColonSign error"),
    }
}

#[test]
fn test_parse_line_with_empty_value() {
    let footer_lines = vec!["status-code equals:"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_err());
    match result.unwrap_err() {
        GreqFooterError::LineHasNoColonSign => {},
        _ => panic!("Expected LineHasNoColonSign error"),
    }
}

#[test]
fn test_parse_or_not_at_beginning() {
    let footer_lines = vec!["not or status-code equals: 200"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_err());
    match result.unwrap_err() {
        GreqFooterError::TheKeywordOrNotInTheBeginning => {},
        _ => panic!("Expected TheKeywordOrNotInTheBeginning error"),
    }
}

#[test]
fn test_parse_multiple_not_keywords() {
    let footer_lines = vec!["not not status-code equals: 200"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_err());
    match result.unwrap_err() {
        GreqFooterError::TheKeywordNotAppearsMoreThanOnce => {},
        _ => panic!("Expected TheKeywordNotAppearsMoreThanOnce error"),
    }
}

#[test]
fn test_parse_invalid_header_key_empty() {
    let footer_lines = vec!["headers. equals: value"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_err());
    match result.unwrap_err() {
        GreqFooterError::InvalidHeaderKey { header_key } => {
            assert_eq!(header_key, "headers.");
        },
        _ => panic!("Expected InvalidHeaderKey error"),
    }
}

#[test]
fn test_parse_invalid_key() {
    let footer_lines = vec!["unknown-keyword: value"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_err());
    match result.unwrap_err() {
        GreqFooterError::InvalidKey { key } => {
            assert_eq!(key, "unknown-keyword");
        },
        _ => panic!("Expected InvalidKey error"),
    }
}

#[test]
fn test_parse_whitespace_handling() {
    let footer_lines = vec![
        "  status-code   equals  :   200  ",
        "\theaders.content-type\tequals\t:\tapplication/json\t"
    ];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 2);
    assert_eq!(footer.conditions[0].value, "200");
    assert_eq!(footer.conditions[1].value, "application/json");
}

#[test]
fn test_parse_case_insensitive_keywords() {
    let footer_lines = vec![
        "STATUS-CODE EQUALS: 200",
        "Or Not Response-Body Contains: error"
    ];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 2);
    assert_eq!(footer.conditions[0].key, "status-code");
    assert!(footer.conditions[1].has_or);
    assert!(footer.conditions[1].has_not);
}

#[test]
fn test_parse_mixed_valid_and_empty_lines() {
    let footer_lines = vec![
        "",
        "status-code equals: 200",
        "   ",
        "response-body contains: success",
        ""
    ];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 2);
}

#[test]
fn test_parse_header_with_dashes() {
    let footer_lines = vec!["headers.x-api-key equals: secret123"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    assert_eq!(footer.conditions[0].key, "x-api-key");
}

#[test]
fn test_parse_numeric_values() {
    let footer_lines = vec![
        "status-code greater-than: 199",
        "status-code less-than: 300"
    ];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 2);
    assert_eq!(footer.conditions[0].value, "199");
    assert_eq!(footer.conditions[1].value, "300");
}

#[test]
fn test_parse_regex_pattern() {
    let footer_lines = vec!["response-body matches-regex: ^[0-9]+$"];
    let result = GreqFooter::parse(&footer_lines);
    assert!(result.is_ok());
    let footer = result.unwrap();
    assert_eq!(footer.conditions.len(), 1);
    assert_eq!(footer.conditions[0].operator, ConditionOperator::MatchesRegex);
    assert_eq!(footer.conditions[0].value, "^[0-9]+$");
}
