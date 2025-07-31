use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GreqResponse {
    pub status_code: u16,
    pub reason_phrase: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub response_milliseconds: u64,
    pub evaluation_result: bool,
}

impl GreqResponse {
    /// gets the value of a variable from the response
    /// the placeholder must start with "dependency." or "dep."
    pub fn get_var(&self, var: &str) -> String {
        let mut the_var = var;
        if the_var.is_empty() {
            return String::new();
        }

        // check if the variable starts with "dependency." or "dep."
        let dependency_prefix1 = "dependency.";
        let dependency_prefix2 = "dep.";
        if the_var.starts_with(dependency_prefix1) {
            the_var = &var[dependency_prefix1.len()..];
        } else if the_var.starts_with(dependency_prefix2) {
            the_var = &var[dependency_prefix2.len()..];
        } else {
            return String::new(); // If it doesn't start with "dependency.", return empty
        }

        match the_var {
            "status_code" => self.status_code.to_string(),
            "reason_phrase" => self.reason_phrase.clone(),
            "evaluation_result" => if self.evaluation_result { 
                "true".to_string() 
            } else { 
                "false".to_string() 
            },
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
