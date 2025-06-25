
pub const NEW_LINE: &str = "\r\n";
pub const COMMENT_PREFIX: &str = "--";
pub const DEFAULT_DELIMITER_CHAR: char = '=';
pub const DELIMITER_MIN_LENGTH: usize = 4;

pub const DEFAULT_HTTP_VERSION: &str = "HTTP/1.1";

pub const FOOTER_CONDITION_HEADERS_PREFIX: &str = "headers.";
pub const FOOTER_CONDITION_ALLOWED_KEY_WORDS: [&str; 12] = [
    "or", "not", "status-code", "response-body", "equals", "contains",
    "starts-with", "ends-with", "matches-regex", 
    "greater-than", "less-than", "case-sensitive"
];

