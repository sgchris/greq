use greq::greq_object::greq_content::GreqContent;
use greq::greq_object::traits::enrich_with_trait::EnrichWith;
use std::collections::HashMap;
use greq::constants::{DEFAULT_HTTP_VERSION, NEW_LINE};
use greq::greq_object::greq_content::GreqContentError;

#[test]
fn test_parse_minimal_valid_request() {
    let content_lines = vec![
        "GET /index.html HTTP/1.1",
        "Host: example.com"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.method, "GET");
    assert_eq!(result.uri, "/index.html");
    assert_eq!(result.http_version, "HTTP/1.1");
    assert_eq!(result.hostname, "example.com");
    assert_eq!(result.custom_port, None);
    assert_eq!(result.headers.get("Host").unwrap(), "example.com");
    assert_eq!(result.body, "");
}

#[test]
fn test_parse_with_custom_port() {
    let content_lines = vec![
        "GET /api/users HTTP/1.1",
        "Host: api.example.com:8080"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.hostname, "api.example.com");
    assert_eq!(result.custom_port, Some(8080));
    assert_eq!(result.headers.get("Host").unwrap(), "api.example.com:8080");
}

#[test]
fn test_parse_with_default_http_version() {
    let content_lines = vec![
        "POST /submit",
        "Host: example.com"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.method, "POST");
    assert_eq!(result.uri, "/submit");
    assert_eq!(result.http_version, DEFAULT_HTTP_VERSION);
}

#[test]
fn test_parse_with_multiple_headers() {
    let content_lines = vec![
        "POST /api/data HTTP/1.1",
        "Host: api.example.com",
        "Content-Type: application/json",
        "Authorization: Bearer token123",
        "User-Agent: MyApp/1.0"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.headers.len(), 4);
    assert_eq!(result.headers.get("Content-Type").unwrap(), "application/json");
    assert_eq!(result.headers.get("Authorization").unwrap(), "Bearer token123");
    assert_eq!(result.headers.get("User-Agent").unwrap(), "MyApp/1.0");
}

#[test]
fn test_parse_with_body() {
    let content_lines = vec![
        "POST /api/users HTTP/1.1",
        "Host: api.example.com",
        "Content-Type: application/json",
        "",
        "{\"name\": \"John Doe\"}",
        "{\"age\": 30}"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.body, format!("{{\"name\": \"John Doe\"}}{}{{\"age\": 30}}", NEW_LINE));
}

#[test]
fn test_parse_with_empty_body() {
    let content_lines = vec![
        "GET /api/status HTTP/1.1",
        "Host: api.example.com",
        ""
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.body, "");
}

#[test]
fn test_parse_all_valid_http_methods() {
    let methods = ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "TRACE", "CONNECT"];

    for method in &methods {
        let request_line = format!("{} /test HTTP/1.1", method);
        let host_header_line = format!("Host: example.com");
        let content_lines: Vec<&str> = vec![&request_line, &host_header_line];

        let result = GreqContent::parse(&content_lines).unwrap();
        assert_eq!(result.method, *method);
    }
}

#[test]
fn test_parse_whitespace_handling() {
    let content_lines = vec![
        "  GET   /test   HTTP/1.1  ",
        "  Host  :   example.com   ",
        "  Content-Type  :  application/json  "
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.method, "GET");
    assert_eq!(result.uri, "/test");
    assert_eq!(result.hostname, "example.com");
    assert_eq!(result.headers.get("Host").unwrap(), "example.com");
    assert_eq!(result.headers.get("Content-Type").unwrap(), "application/json");
}

#[test]
fn test_parse_case_insensitive_host_header() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "HOST: example.com"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.hostname, "example.com");
}

#[test]
fn test_parse_host_with_port_whitespace() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Host:  example.com : 9090  "
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.hostname, "example.com");
    assert_eq!(result.custom_port, Some(9090));
}

// Error cases

#[test]
fn test_parse_empty_content() {
    let content_lines = vec![];

    let result = GreqContent::parse(&content_lines);

    assert!(matches!(result, Err(GreqContentError::EmptyContent)));
}

#[test]
fn test_parse_single_line_only() {
    let content_lines = vec!["GET /test HTTP/1.1"];

    let result = GreqContent::parse(&content_lines);

    assert!(matches!(result, Err(GreqContentError::EmptyContent)));
}

#[test]
fn test_parse_missing_http_method() {
    let content_lines = vec![
        "",
        "Host: example.com"
    ];

    let result = GreqContent::parse(&content_lines);

    assert!(matches!(result, Err(GreqContentError::MissingHttpMethod)));
}

#[test]
fn test_parse_invalid_http_method() {
    let content_lines = vec![
        "INVALID /test HTTP/1.1",
        "Host: example.com"
    ];

    let result = GreqContent::parse(&content_lines);

    if let Err(GreqContentError::InvalidHttpMethod { method }) = result {
        assert_eq!(method, "INVALID");
    } else {
        panic!("Expected InvalidHttpMethod error");
    }
}

#[test]
fn test_parse_missing_uri() {
    let content_lines = vec![
        "GET",
        "Host: example.com"
    ];

    let result = GreqContent::parse(&content_lines);

    assert!(matches!(result, Err(GreqContentError::MissingUri)));
}

#[test]
fn test_parse_invalid_http_version() {
    let content_lines = vec![
        "GET /test INVALID/1.1",
        "Host: example.com"
    ];

    let result = GreqContent::parse(&content_lines);

    assert!(matches!(result, Err(GreqContentError::InvalidHttpVersion)));
}

#[test]
fn test_parse_missing_host_header() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Content-Type: application/json"
    ];

    let result = GreqContent::parse(&content_lines);

    assert!(matches!(result, Err(GreqContentError::MissingHost)));
}

#[test]
fn test_parse_empty_hostname() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Host: "
    ];

    let result = GreqContent::parse(&content_lines);

    assert!(matches!(result, Err(GreqContentError::MissingHost)));
}

#[test]
fn test_parse_invalid_port() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Host: example.com:invalid"
    ];

    let result = GreqContent::parse(&content_lines);

    if let Err(GreqContentError::InvalidPort { line }) = result {
        assert!(line.contains("Host: example.com:invalid"));
    } else {
        panic!("Expected InvalidPort error");
    }
}

#[test]
fn test_parse_port_out_of_range() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Host: example.com:99999"
    ];

    let result = GreqContent::parse(&content_lines);

    assert!(matches!(result, Err(GreqContentError::InvalidPort { .. })));
}

#[test]
fn test_parse_invalid_header_line() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Host: example.com",
        "Invalid header without colon"
    ];

    let result = GreqContent::parse(&content_lines);

    if let Err(GreqContentError::InvalidHeaderLine { line }) = result {
        assert_eq!(line, "Invalid header without colon");
    } else {
        panic!("Expected InvalidHeaderLine error");
    }
}

// Edge cases

#[test]
fn test_parse_root_uri() {
    let content_lines = vec![
        "GET / HTTP/1.1",
        "Host: example.com"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.uri, "/");
}

#[test]
fn test_parse_uri_with_query_params() {
    let content_lines = vec![
        "GET /search?q=rust&page=1 HTTP/1.1",
        "Host: example.com"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.uri, "/search?q=rust&page=1");
}

#[test]
fn test_parse_long_uri() {
    let long_path = "/".to_string() + &"a".repeat(1000);
    let request_line = format!("GET {} HTTP/1.1", long_path);
    let host_header = "Host: example.com";
    let content_lines: Vec<&str> = vec![
        &request_line,
        host_header
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.uri, long_path);
}

#[test]
fn test_parse_header_with_empty_value() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Host: example.com",
        "X-Empty-Header:"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.headers.get("X-Empty-Header").unwrap(), "");
}

#[test]
fn test_parse_header_with_colon_in_value() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Host: example.com",
        "X-URL: https://example.com:443/path"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.headers.get("X-URL").unwrap(), "https://example.com:443/path");
}

#[test]
fn test_parse_multiple_empty_lines_in_body() {
    let content_lines = vec![
        "POST /test HTTP/1.1",
        "Host: example.com",
        "",
        "First line",
        "",
        "",
        "Last line"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    let expected_body = format!("First line{}{}{}Last line", NEW_LINE, NEW_LINE, NEW_LINE);
    assert_eq!(result.body, expected_body);
}

#[test]
fn test_parse_body_only_empty_lines() {
    let content_lines = vec![
        "POST /test HTTP/1.1",
        "Host: example.com",
        "",
        "",
        ""
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    let expected_body = format!("{}", NEW_LINE);
    assert_eq!(result.body, expected_body);
}

#[test]
fn test_parse_http_version_variations() {
    let versions = ["HTTP/1.0", "HTTP/1.1", "HTTP/2.0"];

    for version in &versions {
        let request_line = format!("GET /test {}", version);
        let host_header = "Host: example.com";
        let content_lines = vec![
            &request_line,
            host_header
        ];

        let result = GreqContent::parse(&content_lines).unwrap();
        assert_eq!(result.http_version, *version);
    }
}

#[test]
fn test_parse_header_case_preservation() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Host: example.com",
        "Content-Type: application/json",
        "X-Custom-Header: value"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    // Headers should preserve their original case
    assert!(result.headers.contains_key("Content-Type"));
    assert!(result.headers.contains_key("X-Custom-Header"));
    assert!(!result.headers.contains_key("content-type"));
}

#[test]
fn test_parse_host_header_among_many() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "User-Agent: Test",
        "Host: example.com:3000",
        "Accept: */*"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.hostname, "example.com");
    assert_eq!(result.custom_port, Some(3000));
}

#[test]
fn test_parse_zero_port() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Host: example.com:0"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.custom_port, Some(0));
}

#[test]
fn test_parse_max_port() {
    let content_lines = vec![
        "GET /test HTTP/1.1",
        "Host: example.com:65535"
    ];

    let result = GreqContent::parse(&content_lines).unwrap();

    assert_eq!(result.custom_port, Some(65535));
}

// Tests for method validation
#[test]
fn test_method_is_valid() {
    assert!(GreqContent::method_is_valid("GET"));
    assert!(GreqContent::method_is_valid("POST"));
    assert!(GreqContent::method_is_valid("DELETE"));
    assert!(!GreqContent::method_is_valid("INVALID"));
    assert!(!GreqContent::method_is_valid("get")); // case sensitive
}

// Tests for HTTP version validation
#[test]
fn test_is_valid_http_version() {
    assert!(GreqContent::is_valid_http_version("HTTP/1.0"));
    assert!(GreqContent::is_valid_http_version("HTTP/1.1"));
    assert!(GreqContent::is_valid_http_version("HTTP/2.0"));
    assert!(!GreqContent::is_valid_http_version("HTTP/1"));
    assert!(!GreqContent::is_valid_http_version("HTTP/1.1.1"));
    assert!(!GreqContent::is_valid_http_version("http/1.1"));
    assert!(!GreqContent::is_valid_http_version("INVALID"));
}



#[test]
fn test_enrich_with_empty_self() {
    let mut content_self = GreqContent::default();
    let content_to_merge = GreqContent {
        method: "GET".to_string(),
        hostname: "example.com".to_string(),
        custom_port: Some(443),
        http_version: "HTTP/1.1".to_string(),
        uri: "/test".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Host".to_string(), "example.com".to_string());
            h
        },
        body: "body".to_string(),
    };

    content_self.enrich_with(&content_to_merge).unwrap();

    assert_eq!(content_self.method, "GET");
    assert_eq!(content_self.uri, "/test");
    assert_eq!(content_self.hostname, "example.com");
    assert_eq!(content_self.custom_port, Some(443));
    assert_eq!(content_self.headers.get("Host").unwrap(), "example.com");
    assert_eq!(content_self.body, "body");
}

#[test]
fn test_enrich_with_non_empty_self() {
    let mut content_self = GreqContent {
        method: "GET".to_string(),
        hostname: "base.com".to_string(),
        custom_port: Some(443),
        http_version: "HTTP/1.1".to_string(),
        uri: "/base".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Host".to_string(), "base.com".to_string());
            h
        },
        body: "base body".to_string(),
    };

    let content_to_merge = GreqContent {
        method: "POST".to_string(),
        hostname: "merge.com".to_string(),
        custom_port: Some(8080),
        http_version: "HTTP/1.1".to_string(),
        uri: "/merge".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Host".to_string(), "merge.com".to_string());
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
        body: "merge body".to_string(),
    };

    content_self.enrich_with(&content_to_merge).unwrap();

    // Method, URI, hostname, port and body should be updated
    assert_eq!(content_self.method, "GET");
    assert_eq!(content_self.uri, "/base");
    assert_eq!(content_self.hostname, "base.com");
    assert_eq!(content_self.custom_port, Some(443));
    assert_eq!(content_self.body, "base body");

    // Headers should be merged
    assert_eq!(content_self.headers.get("Host").unwrap(), "base.com");
    assert_eq!(content_self.headers.get("Content-Type").unwrap(), "application/json");
}

#[test]
fn test_enrich_with_headers_only() {
    let mut content_self = GreqContent {
        method: "GET".to_string(),
        hostname: "example.com".to_string(),
        custom_port: Some(443),
        http_version: "HTTP/1.1".to_string(),
        uri: "/test".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Host".to_string(), "example.com".to_string());
            h
        },
        body: String::new(),
    };

    let content_to_merge = GreqContent {
        method: "GET".to_string(),
        hostname: "example.com".to_string(),
        custom_port: Some(443),
        http_version: "HTTP/1.1".to_string(),
        uri: "/test".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
        body: String::new(),
    };

    content_self.enrich_with(&content_to_merge).unwrap();

    // Headers should be merged
    assert_eq!(content_self.headers.get("Host").unwrap(), "example.com");
    assert_eq!(content_self.headers.get("Content-Type").unwrap(), "application/json");
    
    // Other fields should remain the same
    assert_eq!(content_self.method, "GET");
    assert_eq!(content_self.uri, "/test");
    assert_eq!(content_self.hostname, "example.com");
    assert_eq!(content_self.custom_port, Some(443));
    assert_eq!(content_self.body, "");
}
