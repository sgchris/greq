use greq::greq_object::greq::Greq;
use greq::constants::{ DEFAULT_HTTP_PORT, DEFAULT_HTTPS_PORT };

#[test]
fn test_get_full_url_https_default_port() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "example.com".to_string();
    greq.content.port = DEFAULT_HTTPS_PORT;
    greq.content.uri = "/api/users".to_string();

    assert_eq!(greq.get_full_url(), "https://example.com/api/users");
}

#[test]
fn test_get_full_url_http_default_port() {
    let mut greq = Greq::default();
    greq.header.is_http = true;
    greq.content.hostname = "example.com".to_string();
    greq.content.port = DEFAULT_HTTP_PORT;
    greq.content.uri = "/api/users".to_string();

    assert_eq!(greq.get_full_url(), "http://example.com/api/users");
}

#[test]
fn test_get_full_url_https_custom_port() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "example.com".to_string();
    greq.content.port = 8443;
    greq.content.uri = "/api/users".to_string();

    assert_eq!(greq.get_full_url(), "https://example.com:8443/api/users");
}

#[test]
fn test_get_full_url_http_custom_port() {
    let mut greq = Greq::default();
    greq.header.is_http = true;
    greq.content.hostname = "example.com".to_string();
    greq.content.port = 8080;
    greq.content.uri = "/api/users".to_string();

    assert_eq!(greq.get_full_url(), "http://example.com:8080/api/users");
}

#[test]
fn test_get_full_url_empty_uri() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "example.com".to_string();
    greq.content.port = DEFAULT_HTTPS_PORT;
    greq.content.uri = "".to_string();

    assert_eq!(greq.get_full_url(), "https://example.com"); // is_http is stronger than port
}

#[test]
fn test_get_full_url_root_uri() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "example.com".to_string();
    greq.content.port = DEFAULT_HTTPS_PORT;
    greq.content.uri = "/".to_string();

    assert_eq!(greq.get_full_url(), "https://example.com/");
}

#[test]
fn test_get_full_url_uri_with_query_params() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "api.example.com".to_string();
    greq.content.port = DEFAULT_HTTPS_PORT;
    greq.content.uri = "/search?q=rust&limit=10".to_string();

    assert_eq!(greq.get_full_url(), "https://api.example.com/search?q=rust&limit=10");
}

#[test]
fn test_get_full_url_uri_with_fragment() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "docs.example.com".to_string();
    greq.content.port = DEFAULT_HTTPS_PORT;
    greq.content.uri = "/guide#installation".to_string();

    assert_eq!(greq.get_full_url(), "https://docs.example.com/guide#installation");
}

#[test]
fn test_get_full_url_localhost() {
    let mut greq = Greq::default();
    greq.header.is_http = true;
    greq.content.hostname = "localhost".to_string();
    greq.content.port = 3000;
    greq.content.uri = "/api/health".to_string();

    assert_eq!(greq.get_full_url(), "http://localhost:3000/api/health");
}

#[test]
fn test_get_full_url_ip_address() {
    let mut greq = Greq::default();
    greq.header.is_http = true;
    greq.content.hostname = "192.168.1.100".to_string();
    greq.content.port = 8080;
    greq.content.uri = "/status".to_string();

    assert_eq!(greq.get_full_url(), "http://192.168.1.100:8080/status");
}

#[test]
fn test_get_full_url_ipv6_address() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "[::1]".to_string();
    greq.content.port = 9000;
    greq.content.uri = "/api/v1/data".to_string();

    assert_eq!(greq.get_full_url(), "https://[::1]:9000/api/v1/data");
}

#[test]
fn test_get_full_url_subdomain() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "api.v2.example.com".to_string();
    greq.content.port = DEFAULT_HTTPS_PORT;
    greq.content.uri = "/users/123".to_string();

    assert_eq!(greq.get_full_url(), "https://api.v2.example.com/users/123");
}

#[test]
fn test_get_full_url_http_with_https_default_port() {
    let mut greq = Greq::default();
    greq.header.is_http = true;
    greq.content.hostname = "example.com".to_string();
    greq.content.port = DEFAULT_HTTPS_PORT; // 443 but using HTTP
    greq.content.uri = "/api".to_string();

    assert_eq!(greq.get_full_url(), "http://example.com/api"); // "is_http" is stronger
}

#[test]
fn test_get_full_url_https_with_http_default_port() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "example.com".to_string();
    greq.content.port = DEFAULT_HTTP_PORT; // 80 but using HTTPS
    greq.content.uri = "/secure".to_string();

    assert_eq!(greq.get_full_url(), "https://example.com/secure");
}

#[test]
fn test_get_full_url_complex_uri() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "api.example.com".to_string();
    greq.content.port = DEFAULT_HTTPS_PORT;
    greq.content.uri = "/v1/users/123/posts?include=comments&sort=date#latest".to_string();

    assert_eq!(
        greq.get_full_url(), 
        "https://api.example.com/v1/users/123/posts?include=comments&sort=date#latest"
    );
}

#[test]
fn test_get_full_url_uri_without_leading_slash() {
    let mut greq = Greq::default();
    greq.header.is_http = false;
    greq.content.hostname = "example.com".to_string();
    greq.content.port = DEFAULT_HTTPS_PORT;
    greq.content.uri = "api/users".to_string(); // No leading slash

    assert_eq!(greq.get_full_url(), "https://example.com/api/users");
}

#[test]
fn test_get_full_url_port_edge_cases() {
    // Test port 1 (minimum valid port)
    let mut greq = Greq::default();
    greq.header.is_http = true;
    greq.content.hostname = "example.com".to_string();
    greq.content.port = 1;
    greq.content.uri = "/test".to_string();

    assert_eq!(greq.get_full_url(), "http://example.com:1/test");

    // Test port 65535 (maximum valid port)
    greq.content.port = 65535;
    assert_eq!(greq.get_full_url(), "http://example.com:65535/test");
}
