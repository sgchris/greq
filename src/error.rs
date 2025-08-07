use thiserror::Error;

/// Represents all possible errors that can occur in Greq
#[derive(Error, Debug)]
pub enum GreqError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Dependency error: {0}")]
    Dependency(String),
    
    #[error("Placeholder error: {0}")]
    Placeholder(String),
    
    #[error("Condition evaluation failed: {0}")]
    ConditionFailed(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Timeout error: request timed out")]
    Timeout,
}

/// Type alias for Result with GreqError
pub type Result<T> = std::result::Result<T, GreqError>;
