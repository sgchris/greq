use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GreqResponse {
    pub status_code: u16,
    pub reason_phrase: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub response_milliseconds: u64,
}

impl GreqResponse {
    pub fn get_var(&self, var: &str) -> String {
        match var {
            "status_code" => self.status_code.to_string(),
            "reason_phrase" => self.reason_phrase.clone(),
            "headers" => serde_json::to_string(&self.headers).unwrap(),
            h if h.starts_with("header.") => {
                let header_name = &h[7..]; // Remove "header." prefix
                if self.headers.contains_key(header_name) {
                    self.headers[header_name].clone()
                } else {
                    String::new()
                }
            }
            "body" => self.body.clone().unwrap_or_default().clone(),
            "response_milliseconds" => self.response_milliseconds.to_string(),
            _ => String::new(),
        }
    }
}
