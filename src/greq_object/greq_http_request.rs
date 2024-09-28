use std::collections::HashMap;

// Single request properties
#[derive(Debug, Default)]
pub struct GreqHttpRequest {
    pub method: String,
    pub hostname: String,
    pub http_version: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub content: String,
}
