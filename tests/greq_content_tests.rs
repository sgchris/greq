
use greq::greq_object::{
    greq_content::{
        GreqContentError,
        GreqContent,
    },
    traits::enrich_with_trait::EnrichWith,
};
use std::collections::HashMap;

#[test]
fn test_parse_complete_success_scenarios() {
    // Test 1: Minimal valid request
    let content_lines = vec!["GET /test HTTP/1.1", "Host: example.com"];
    let result = GreqContent::parse(&content_lines, None, None).unwrap();
    assert_eq!(result.method, "GET");
    assert_eq!(result.uri, "/test");
    assert_eq!(result.hostname, "example.com");
    assert_eq!(result.custom_port, None);

    // Test 2: Complete request with port, headers, and body
    let content_lines = vec![
        "POST /api/users HTTP/1.1",
        "Host: api.example.com:8080",
        "Content-Type: application/json",
        "Authorization: Bearer token123",
        "",
        "{\"name\": \"John\"}",
        "{\"age\": 30}"
    ];
    let result = GreqContent::parse(&content_lines, None, None).unwrap();
    assert_eq!(result.method, "POST");
    assert_eq!(result.hostname, "api.example.com");
    assert_eq!(result.custom_port, Some(8080));
    assert_eq!(result.headers.len(), 3);
    assert!(result.body.contains("John") && result.body.contains("30"));

    // Test 3: Default HTTP version and edge cases
    let content_lines = vec![
        "DELETE /",
        "HOST: localhost:65535"  // Case insensitive, max port, root URI
    ];
    let result = GreqContent::parse(&content_lines, None, None).unwrap();
    assert_eq!(result.http_version, "HTTP/1.1");
    assert_eq!(result.uri, "/");
    assert_eq!(result.custom_port, Some(65535));

    // Test 4: Empty content should return default
    let empty_lines = vec![];
    let result = GreqContent::parse(&empty_lines, None, None).unwrap();
    assert_eq!(result.method, "");
    assert_eq!(result.hostname, "");
}

#[test]
fn test_parse_all_error_scenarios() {
    // Test invalid HTTP method
    let content_lines = vec!["INVALID /test HTTP/1.1", "Host: example.com"];
    let result = GreqContent::parse(&content_lines, None, None);
    assert!(matches!(result, Err(GreqContentError::InvalidHttpMethod { .. })));

    // Test missing URI
    let content_lines = vec!["GET", "Host: example.com"];
    let result = GreqContent::parse(&content_lines, None, None);
    assert!(matches!(result, Err(GreqContentError::InvalidRequestLine { .. })));

    // Test invalid HTTP version
    let content_lines = vec!["GET /test INVALID/1.1", "Host: example.com"];
    let result = GreqContent::parse(&content_lines, None, None);
    assert!(matches!(result, Err(GreqContentError::InvalidHttpVersion)));

    // Test invalid port
    let content_lines = vec!["GET /test HTTP/1.1", "Host: example.com:invalid"];
    let result = GreqContent::parse(&content_lines, None, None);
    assert!(matches!(result, Err(GreqContentError::InvalidPort { .. })));

    // Test port out of range
    let content_lines = vec!["GET /test HTTP/1.1", "Host: example.com:99999"];
    let result = GreqContent::parse(&content_lines, None, None);
    assert!(matches!(result, Err(GreqContentError::InvalidPort { .. })));

    // Test invalid header line
    let content_lines = vec!["GET /test HTTP/1.1", "Host: example.com", "Invalid header without colon"];
    let result = GreqContent::parse(&content_lines, None, None);
    assert!(matches!(result, Err(GreqContentError::InvalidHeaderLine { .. })));

    // Test invalid URI characters
    let content_lines = vec!["GET /test<>invalid HTTP/1.1", "Host: example.com"];
    let result = GreqContent::parse(&content_lines, None, None);
    assert!(matches!(result, Err(GreqContentError::InvalidUri { .. })));

    // Test empty hostname
    let content_lines = vec!["GET /test HTTP/1.1", "Host: "];
    let result = GreqContent::parse(&content_lines, None, None);
    assert!(matches!(result, Err(GreqContentError::MissingHost)));
}

#[test]
fn test_validation_methods() {
    // Test HTTP method validation
    assert!(GreqContent::is_valid_http_method("GET"));
    assert!(GreqContent::is_valid_http_method("POST"));
    assert!(GreqContent::is_valid_http_method("DELETE"));
    assert!(GreqContent::is_valid_http_method("CONNECT"));
    assert!(!GreqContent::is_valid_http_method("INVALID"));
    assert!(!GreqContent::is_valid_http_method("get")); // case sensitive

    // Test HTTP version validation
    assert!(GreqContent::is_valid_http_version("HTTP/1.0"));
    assert!(GreqContent::is_valid_http_version("HTTP/1.1"));
    assert!(GreqContent::is_valid_http_version("HTTP/2.0"));
    assert!(!GreqContent::is_valid_http_version("HTTP/1"));
    assert!(!GreqContent::is_valid_http_version("HTTP/1.1.1"));
    assert!(!GreqContent::is_valid_http_version("http/1.1"));
    assert!(!GreqContent::is_valid_http_version("INVALID"));
}

#[test]
fn test_enrich_with_comprehensive() {
    // Test 1: Enrich empty content with complete content
    let mut empty_content = GreqContent::default();
    let complete_content = GreqContent {
        method: "POST".to_string(),
        hostname: "api.example.com".to_string(),
        custom_port: Some(8080),
        http_version: "HTTP/1.1".to_string(),
        uri: "/api/users".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Host".to_string(), "api.example.com:8080".to_string());
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
        body: "{\"name\": \"John\"}".to_string(),
    };

    empty_content.enrich_with(&complete_content).unwrap();
    
    assert_eq!(empty_content.method, ""); // method is not enriched
    assert_eq!(empty_content.hostname, "api.example.com");
    assert_eq!(empty_content.custom_port, Some(8080));
    assert_eq!(empty_content.headers.get("Content-Type").unwrap(), "application/json");
    assert_eq!(empty_content.body, "{\"name\": \"John\"}");

    // Test 2: Enrich existing content (should not override existing values)
    let mut existing_content = GreqContent {
        method: "GET".to_string(),
        hostname: "existing.com".to_string(),
        custom_port: Some(443),
        http_version: "HTTP/1.0".to_string(),
        uri: "/existing".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Host".to_string(), "existing.com".to_string());
            h.insert("User-Agent".to_string(), "MyApp".to_string());
            h
        },
        body: "existing body".to_string(),
    };

    let merge_content = GreqContent {
        method: "POST".to_string(),
        hostname: "new.com".to_string(),
        custom_port: Some(8080),
        http_version: "HTTP/1.1".to_string(),
        uri: "/new".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h.insert("Authorization".to_string(), "Bearer token".to_string());
            h
        },
        body: "new body".to_string(),
    };

    existing_content.enrich_with(&merge_content).unwrap();

    // Existing values should be preserved
    assert_eq!(existing_content.hostname, "existing.com");
    assert_eq!(existing_content.custom_port, Some(443));
    assert_eq!(existing_content.body, "existing body");
    
    // New headers should be added, existing headers preserved
    assert_eq!(existing_content.headers.get("User-Agent").unwrap(), "MyApp");
    assert_eq!(existing_content.headers.get("Content-Type").unwrap(), "application/json");
    assert_eq!(existing_content.headers.len(), 3);

    // Test 3: Partial enrichment
    let mut partial_content = GreqContent {
        hostname: "partial.com".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Existing".to_string(), "value".to_string());
            h
        },
        ..Default::default()
    };

    let enrich_content = GreqContent {
        custom_port: Some(9000),
        body: "enriched body".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("New-Header".to_string(), "new-value".to_string());
            h
        },
        ..Default::default()
    };

    partial_content.enrich_with(&enrich_content).unwrap();

    assert_eq!(partial_content.hostname, "partial.com"); // preserved
    assert_eq!(partial_content.custom_port, Some(9000)); // enriched
    assert_eq!(partial_content.body, "enriched body"); // enriched
    assert_eq!(partial_content.headers.len(), 2); // both headers present
}
