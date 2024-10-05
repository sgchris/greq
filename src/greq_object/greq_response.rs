use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct GreqResponse {
    pub status_code: u16,
    pub reason_phrase: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl GreqResponse {
    pub fn new(status_code: u16, reason_phrase: &str, headers: HashMap<String, String>, body: Option<String>) -> Self {
        GreqResponse {
            status_code,
            reason_phrase: reason_phrase.to_string(),
            headers,
            body,
        }
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn set_body(&mut self, body_content: String) {
        self.body = Some(body_content);
    }

    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    pub fn is_redirect(&self) -> bool {
        self.status_code >= 300 && self.status_code < 400
    }

    pub fn is_error(&self) -> bool {
        self.status_code >= 400
    }
}
