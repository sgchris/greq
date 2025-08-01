use greq::greq_object::greq_response::GreqResponse;
use std::collections::HashMap;

#[test]
fn test_get_var_basic_fields() {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("authorization".to_string(), "Bearer token123".to_string());

    let response = GreqResponse {
        status_code: 200,
        reason_phrase: "OK".to_string(),
        headers,
        body: Some("response body".to_string()),
        response_milliseconds: 150,
        evaluation_result: false,
    };

    assert_eq!(response.get_var("status_code"), "200");
    assert_eq!(response.get_var("reason_phrase"), "OK");
    assert_eq!(response.get_var("body"), "response body");
    assert_eq!(response.get_var("response_milliseconds"), "150");

    // Test headers serialization
    let headers_json = response.get_var("headers");
    assert!(headers_json.contains("content-type"));
    assert!(headers_json.contains("application/json"));
}

#[test]
fn test_get_var_header_access() {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("x-custom-header".to_string(), "custom-value".to_string());

    let response = GreqResponse {
        status_code: 404,
        reason_phrase: "Not Found".to_string(),
        headers,
        body: None,
        response_milliseconds: 75,
        evaluation_result: false,
    };

    // Test existing headers
    assert_eq!(response.get_var("header.content-type"), "application/json");
    assert_eq!(response.get_var("header.x-custom-header"), "custom-value");

    // Test non-existent header
    assert_eq!(response.get_var("header.non-existent"), "");

    // Test body when None
    assert_eq!(response.get_var("body"), "");
}

#[test]
fn test_get_var_unknown_variables() {
    let response = GreqResponse {
        status_code: 500,
        reason_phrase: "Internal Server Error".to_string(),
        headers: HashMap::new(),
        body: Some("error message".to_string()),
        response_milliseconds: 1000,
        evaluation_result: false,
    };

    // Test unknown variable names
    assert_eq!(response.get_var("unknown_field"), "");
    assert_eq!(response.get_var(""), "");
    assert_eq!(response.get_var("header."), "");
    assert_eq!(response.get_var("status"), "");
    assert_eq!(response.get_var("body_length"), "");
}

