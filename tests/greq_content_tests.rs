use greq::greq_object::greq_content::GreqContent;
use greq::greq_object::greq_http_request::GreqHttpRequest;
use greq::greq_object::traits::enrich_with_trait::EnrichWith;
use std::collections::HashMap;

#[test]
fn test_enrich_with_empty_self() {
    let mut content_self = GreqContent::default();
    let content_to_merge = GreqContent {
        original_string: "GET /test HTTP/1.1\r\nHost: example.com\r\n\r\nbody".to_string(),
        http_request: GreqHttpRequest {
            method: "GET".to_string(),
            uri: "/test".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Host".to_string(), "example.com".to_string());
                h
            },
            content: "body".to_string(),
            ..Default::default()
        },
    };

    content_self.enrich_with(&content_to_merge).unwrap();

    assert_eq!(content_self.original_string, content_to_merge.original_string);
    assert_eq!(content_self.http_request.method, "GET");
    assert_eq!(content_self.http_request.uri, "/test");
    assert_eq!(content_self.http_request.headers.get("Host").unwrap(), "example.com");
    assert_eq!(content_self.http_request.content, "body");
}

#[test]
fn test_enrich_with_non_empty_self() {
    let mut content_self = GreqContent {
        original_string: "GET /base HTTP/1.1\r\nHost: base.com\r\n\r\nbase body".to_string(),
        http_request: GreqHttpRequest {
            method: "GET".to_string(),
            uri: "/base".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Host".to_string(), "base.com".to_string());
                h
            },
            content: "base body".to_string(),
            ..Default::default()
        },
    };

    let content_to_merge = GreqContent {
        original_string: "POST /merge HTTP/1.1\r\nHost: merge.com\r\nContent-Type: application/json\r\n\r\nmerge body".to_string(),
        http_request: GreqHttpRequest {
            method: "POST".to_string(),
            uri: "/merge".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Host".to_string(), "merge.com".to_string());
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
            content: "merge body".to_string(),
            ..Default::default()
        },
    };

    content_self.enrich_with(&content_to_merge).unwrap();

    // Method, URI, and content should be updated
    assert_eq!(content_self.http_request.method, "POST");
    assert_eq!(content_self.http_request.uri, "/merge");
    assert_eq!(content_self.http_request.content, "merge body");

    // Headers should be merged
    assert_eq!(content_self.http_request.headers.get("Host").unwrap(), "merge.com");
    assert_eq!(content_self.http_request.headers.get("Content-Type").unwrap(), "application/json");

    // Original string should be updated to reflect the merged state
    assert!(content_self.original_string.contains("POST /merge"));
    assert!(content_self.original_string.contains("Host: merge.com"));
    assert!(content_self.original_string.contains("Content-Type: application/json"));
    assert!(content_self.original_string.contains("merge body"));
}

#[test]
fn test_enrich_with_headers_only() {
    let mut content_self = GreqContent {
        original_string: "GET /test HTTP/1.1\r\nHost: example.com".to_string(),
        http_request: GreqHttpRequest {
            method: "GET".to_string(),
            uri: "/test".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Host".to_string(), "example.com".to_string());
                h
            },
            content: String::new(),
            ..Default::default()
        },
    };

    let content_to_merge = GreqContent {
        original_string: "GET /test HTTP/1.1\r\nContent-Type: application/json".to_string(),
        http_request: GreqHttpRequest {
            method: "GET".to_string(),
            uri: "/test".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
            content: String::new(),
            ..Default::default()
        },
    };

    content_self.enrich_with(&content_to_merge).unwrap();

    // Headers should be merged
    assert_eq!(content_self.http_request.headers.get("Host").unwrap(), "example.com");
    assert_eq!(content_self.http_request.headers.get("Content-Type").unwrap(), "application/json");

    // Original string should be updated to reflect the merged state
    assert!(content_self.original_string.contains("Host: example.com"));
    assert!(content_self.original_string.contains("Content-Type: application/json"));
} 