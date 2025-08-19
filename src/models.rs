use std::collections::HashMap;
use std::time::Duration;

/// Represents a complete Greq test file with all its sections
#[derive(Debug, Clone)]
pub struct GreqFile {
    pub header: Header,
    pub content: Content,
    pub footer: Footer,
    pub file_path: String,
}

/// Header section containing metadata and execution properties
#[derive(Debug, Clone)]
pub struct Header {
    pub project: Option<String>,
    pub is_http: bool,
    pub delimiter: String,
    pub extends: Option<String>,
    pub number_of_retries: u32,
    pub depends_on: Option<String>,
    pub timeout: Option<Duration>,
    pub allow_dependency_failure: bool,
    pub show_warnings: bool,
}

/// Content section representing the HTTP request
#[derive(Debug, Clone)]
pub struct Content {
    pub request_line: RequestLine,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// HTTP request line (method, URI, version)
#[derive(Debug, Clone)]
pub struct RequestLine {
    pub method: String,
    pub uri: String,
    pub version: String,
}

/// Footer section containing response validation conditions
#[derive(Debug, Clone, Default)]
pub struct Footer {
    pub conditions: Vec<Condition>,
}

/// A single validation condition
#[derive(Debug, Clone)]
pub struct Condition {
    pub is_or: bool,
    pub is_not: bool,
    pub key: ConditionKey,
    pub operator: Operator,
    pub case_sensitive: bool,
    pub value: String,
}

/// The key part of a condition (what to evaluate)
#[derive(Debug, Clone)]
pub enum ConditionKey {
    StatusCode,
    Headers,
    Header(String),
    ResponseBody,
    ResponseBodyPath(String),
    Latency,
}

/// Comparison operators for conditions
#[derive(Debug, Clone)]
pub enum Operator {
    Equals,
    Contains,
    MatchesRegex,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    StartsWith,
    EndsWith,
    Exists,
}

/// HTTP response data
#[derive(Debug, Clone)]
pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub latency: Duration,
}

/// Execution result for a single Greq file
#[derive(Debug)]
pub struct ExecutionResult {
    pub file_path: String,
    pub success: bool,
    pub response: Option<Response>,
    pub failed_conditions: Vec<String>,
    pub error: Option<String>,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            project: None,
            is_http: false,
            delimiter: "=".to_string(),
            extends: None,
            number_of_retries: 0,
            depends_on: None,
            timeout: None,
            allow_dependency_failure: true, // Changed default to true - allow dependencies to fail by default
            show_warnings: true,
        }
    }
}

impl Default for Content {
    fn default() -> Self {
        Self {
            request_line: RequestLine {
                method: "GET".to_string(),
                uri: "/".to_string(),
                version: "HTTP/1.1".to_string(),
            },
            headers: HashMap::new(),
            body: None,
        }
    }
}
