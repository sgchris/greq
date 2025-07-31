use greq::greq_object::greq::Greq;
use greq::greq_object::{
    greq_header::{GreqHeader},
    greq_content::{
        GreqContent,
    },
    greq_footer::{
        GreqFooter
    },
};

fn create_test_greq(hostname: &str, uri: &str, custom_port: Option<u16>, is_http: bool, method: &str) -> Greq {
    Greq {
        file_contents: String::new(),
        sections_delimiter: '=',
        header: GreqHeader {
            is_http,
            ..Default::default()
        },
        content: GreqContent {
            hostname: hostname.to_string(),
            uri: uri.to_string(),
            custom_port,
            method: method.to_string(),
            ..Default::default()
        },
        footer: GreqFooter::default(),
    }
}

#[test]
fn test_get_full_url_basic_scenarios() {
    // HTTPS with path
    let greq = create_test_greq("api.example.com", "users/123", None, false, "GET");
    assert_eq!(greq.get_full_url(), "https://api.example.com/users/123");

    // HTTP with custom port
    let greq = create_test_greq("localhost", "api/health", Some(8080), true, "GET");
    assert_eq!(greq.get_full_url(), "http://localhost:8080/api/health");

    // HTTPS with custom port
    let greq = create_test_greq("secure.api.com", "v1/data", Some(443), false, "POST");
    assert_eq!(greq.get_full_url(), "https://secure.api.com:443/v1/data");
}

#[test]
fn test_get_full_url_uri_edge_cases() {
    // Empty URI
    let greq = create_test_greq("example.com", "", None, false, "GET");
    assert_eq!(greq.get_full_url(), "https://example.com");

    // URI with leading slash
    let greq = create_test_greq("api.test.com", "/users", None, false, "POST");
    assert_eq!(greq.get_full_url(), "https://api.test.com/users");

    // URI with multiple leading slashes
    let greq = create_test_greq("api.test.com", "///users", None, false, "POST");
    assert_eq!(greq.get_full_url(), "https://api.test.com/users");

    // URI without leading slash
    let greq = create_test_greq("api.test.com", "users/123/posts", None, false, "GET");
    assert_eq!(greq.get_full_url(), "https://api.test.com/users/123/posts");
}

#[test]
fn test_get_full_url_complex_scenarios() {
    // URI with query parameters and fragments (edge case)
    let greq = create_test_greq("search.api.com", "search?q=test&limit=10#results", Some(9000), true, "GET");
    assert_eq!(greq.get_full_url(), "http://search.api.com:9000/search?q=test&limit=10#results");

    // Very long URI
    let long_uri = "very/long/path/with/many/segments/that/goes/on/and/on/endpoint";
    let greq = create_test_greq("deep.api.com", long_uri, None, false, "GET");
    assert_eq!(greq.get_full_url(), format!("https://deep.api.com/{}", long_uri));

    // URI with special characters
    let greq = create_test_greq("api.example.com", "users/john@doe.com/profile", None, false, "GET");
    assert_eq!(greq.get_full_url(), "https://api.example.com/users/john@doe.com/profile");
}

#[test]
fn test_request_can_send_body_all_methods() {
    // Methods that CAN send body
    assert!(create_test_greq("test.com", "users", None, false, "POST").request_can_send_body());
    assert!(create_test_greq("test.com", "users", None, false, "PUT").request_can_send_body());
    assert!(create_test_greq("test.com", "users", None, false, "PATCH").request_can_send_body());

    // Methods that CANNOT send body
    assert!(!create_test_greq("test.com", "users", None, false, "GET").request_can_send_body());
    assert!(!create_test_greq("test.com", "users", None, false, "DELETE").request_can_send_body());
    assert!(!create_test_greq("test.com", "users", None, false, "HEAD").request_can_send_body());
    assert!(!create_test_greq("test.com", "users", None, false, "OPTIONS").request_can_send_body());

    // Case sensitivity test
    assert!(create_test_greq("test.com", "users", None, false, "post").request_can_send_body());
    assert!(create_test_greq("test.com", "users", None, false, "Put").request_can_send_body());
    assert!(!create_test_greq("test.com", "users", None, false, "get").request_can_send_body());
}

#[test]
fn test_should_remove_header_standard_cases() {
    // Headers that SHOULD be removed (case insensitive)
    assert!(Greq::should_remove_header_for_reqwest("host"));
    assert!(Greq::should_remove_header_for_reqwest("HOST"));
    assert!(Greq::should_remove_header_for_reqwest("Host"));

    assert!(Greq::should_remove_header_for_reqwest("connection"));
    assert!(Greq::should_remove_header_for_reqwest("keep-alive"));
    assert!(Greq::should_remove_header_for_reqwest("content-length"));
    assert!(Greq::should_remove_header_for_reqwest("transfer-encoding"));

    // Headers that should NOT be removed
    assert!(!Greq::should_remove_header_for_reqwest("authorization"));
    assert!(!Greq::should_remove_header_for_reqwest("content-type"));
    assert!(!Greq::should_remove_header_for_reqwest("accept"));
    assert!(!Greq::should_remove_header_for_reqwest("user-agent"));
}

#[test]
fn test_should_remove_header_proxy_and_http2() {
    // Proxy headers
    assert!(Greq::should_remove_header_for_reqwest("proxy-connection"));
    assert!(Greq::should_remove_header_for_reqwest("proxy-authorization"));
    assert!(Greq::should_remove_header_for_reqwest("Proxy-Connection"));

    // HTTP/2 pseudo headers
    assert!(Greq::should_remove_header_for_reqwest(":method"));
    assert!(Greq::should_remove_header_for_reqwest(":path"));
    assert!(Greq::should_remove_header_for_reqwest(":scheme"));
    assert!(Greq::should_remove_header_for_reqwest(":authority"));

    // Similar looking but allowed headers
    assert!(!Greq::should_remove_header_for_reqwest("x-proxy-auth"));
    assert!(!Greq::should_remove_header_for_reqwest("method"));
    assert!(!Greq::should_remove_header_for_reqwest("custom-connection"));
}

#[test]
fn test_should_remove_header_edge_cases() {
    // Mixed case variations
    assert!(Greq::should_remove_header_for_reqwest("CoNtEnT-LeNgTh"));
    assert!(Greq::should_remove_header_for_reqwest("tRaNsFeR-eNcOdInG"));

    // Empty string and whitespace (should not be removed)
    assert!(!Greq::should_remove_header_for_reqwest(""));
    assert!(!Greq::should_remove_header_for_reqwest(" "));

    // Headers with similar names but different
    assert!(!Greq::should_remove_header_for_reqwest("content-type"));
    assert!(!Greq::should_remove_header_for_reqwest("host-header"));
    assert!(!Greq::should_remove_header_for_reqwest("my-host"));

    // Very long header name
    let long_header = "very-long-custom-header-name-that-should-not-be-removed";
    assert!(!Greq::should_remove_header_for_reqwest(long_header));
}
