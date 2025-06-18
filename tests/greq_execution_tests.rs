use std::str::FromStr;
use greq::greq_object::greq::Greq;
use greq::greq_object::greq::GreqErrorCodes;
use tokio;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_basic_request() {
    let mock_server = MockServer::start().await;
    
    // Setup mock response
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/health"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("healthy"))
        .mount(&mock_server)
        .await;

    let input = format!(r#"project: test-project
output-folder: ./test-output
is-http: true
====

GET /api/health
Host: localhost:{}
Accept: application/json

====

-- Test conditions
status-code equals: 200
response-content contains: "healthy"#,
        mock_server.address().port()
    );

    let parse_result = Greq::from_str(&input);
    assert!(parse_result.is_ok(), "Basic request should parse successfully");
    
    let greq = parse_result.unwrap();
    let Ok((result, _response)) = greq.execute().await else {
        panic!("Basic request should execute successfully");
    };
    assert!(result.unwrap_or(false), "Basic request should evaluate successfully");
}

#[tokio::test]
async fn test_headers_request() {
    let mock_server = MockServer::start().await;

    // Setup mock response
    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/api/users"))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("content-type", "application/json")
            .set_body_string("Success: User created"))
        .mount(&mock_server)
        .await;

    let input = format!(r#"project: test-project
output-folder: ./test-output
is-http: true
====

POST /api/users
Host: localhost:{}
Content-Type: application/json
Accept: application/json

{{
    "name": "John Doe",
    "email": "john@example.com"
}}

====

-- Test conditions
headers.content-type equals: application/json
response-content starts-with case-sensitive: "Success"#,
        mock_server.address().port()
    );

    let parse_result = Greq::from_str(&input);
    assert!(parse_result.is_ok(), "Headers request should parse successfully");
    
    let greq = parse_result.unwrap();
    let Ok((result, _response)) = greq.execute().await else {
        panic!("Headers request should execute successfully");
    };
    assert!(result.unwrap_or(false), "Headers request should evaluate successfully");
}

#[tokio::test]
async fn test_regex_request() {
    let mock_server = MockServer::start().await;
    
    // Setup mock response with UUID
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/search"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(r#"{"id": "550e8400-e29b-41d4-a716-446655440000", "status": "success"}"#))
        .mount(&mock_server)
        .await;

    let input = format!(r#"project: test-project
output-folder: ./test-output
is-http: true
====

GET /api/search?q=test
Host: localhost:{}
Accept: application/json

====

-- Test conditions
response-content contains regex: "[0-9a-f]{{8}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{12}}"
not response-content contains: "error"
not response-content contains: "exception"#,
        mock_server.address().port()
    );

    let parse_result = Greq::from_str(&input);
    assert!(parse_result.is_ok(), "Regex request should parse successfully");
    
    let greq = parse_result.unwrap();
    let Ok((result, _response)) = greq.execute().await else {
        panic!("Regex request should execute successfully");
    };
    assert!(result.unwrap_or(false), "Regex request should evaluate successfully");
}

#[tokio::test]
async fn test_or_conditions_request() {
    let mock_server = MockServer::start().await;
    
    // Setup mock response
    Mock::given(wiremock::matchers::method("PUT"))
        .and(wiremock::matchers::path("/api/settings"))
        .respond_with(ResponseTemplate::new(201)
            .set_body_string("Settings updated successfully"))
        .mount(&mock_server)
        .await;

    let input = format!(r#"project: test-project
output-folder: ./test-output
is-http: true
====

PUT /api/settings
Host: localhost:{}
Content-Type: application/json
Accept: application/json

{{
    "theme": "dark",
    "notifications": true
}}

====

-- Test conditions
or status-code equals: 200
or status-code equals: 201
response-content ends-with: "updated"#,
        mock_server.address().port()
    );

    let parse_result = Greq::from_str(&input);
    assert!(parse_result.is_ok(), "OR conditions request should parse successfully");
    
    let greq = parse_result.unwrap();
    let Ok((result, _response)) = greq.execute().await else {
        panic!("OR conditions request should execute successfully");
    };
    assert!(result.unwrap_or(false), "OR conditions request should evaluate successfully");
}

#[tokio::test]
async fn test_complex_request() {
    let mock_server = MockServer::start().await;
    
    // Setup mock response
    Mock::given(wiremock::matchers::method("DELETE"))
        .and(wiremock::matchers::path("/api/resources/123"))
        .respond_with(ResponseTemplate::new(204)
            .insert_header("content-length", "0")
            .insert_header("etag", "W/\"123\"")
            .set_body_string(""))
        .mount(&mock_server)
        .await;

    let input = format!(r#"project: test-project
output-folder: ./test-output
is-http: true
====

DELETE /api/resources/123
Host: localhost:{}
Authorization: Bearer token123
Accept: application/json

====

-- Test conditions
status-code equals: 204
headers.content-length equals: 0
not response-content contains: "error"
or headers.etag contains: "W/"
or headers.last-modified contains: "GMT"#,
        mock_server.address().port()
    );

    let parse_result = Greq::from_str(&input);
    assert!(parse_result.is_ok(), "Complex request should parse successfully");
    
    let greq = parse_result.unwrap();
    let Ok((result, _response)) = greq.execute().await else {
        panic!("Complex request should execute successfully");
    };
    assert!(result.unwrap_or(false), "Complex request should evaluate successfully");
}

#[test]
fn test_file_parsing() {
    let files = vec![
        "tests/requests/01_basic_test.greq",
        "tests/requests/02_headers_test.greq",
        "tests/requests/03_regex_test.greq",
        "tests/requests/04_or_conditions_test.greq",
        "tests/requests/05_complex_test.greq"
    ];

    for file in files {
        let parse_result = Greq::from_file(file);
        assert!(parse_result.is_ok(), "File {} should parse successfully", file);
    }
}

#[test]
fn test_invalid_file() {
    let parse_result = Greq::from_file("nonexistent.greq");
    assert!(parse_result.is_err(), "Non-existent file should fail to parse");
    assert_eq!(parse_result.unwrap_err().code, GreqErrorCodes::ReadGreqFileError);
} 
