use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Single request properties
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GreqHttpRequest {
    pub is_http: bool,
    pub certificate: String,
    pub method: String,
    pub hostname: String,
    pub port: u16,
    pub http_version: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub content: String,
}

impl GreqHttpRequest {
    pub fn get_full_url(&self) -> String {
        // Determine the protocol based on whether there's a certificate or if the method is HTTP
        let protocol = if self.is_http { "http" } else { "https" };

        // Determine the default port for the protocol
        let default_port = if protocol == "http" { 80 } else { 443 };

        // Construct the host part, including the port only if it's not the default one
        let host = if self.port != default_port && self.port != 0 {
            format!("{}:{}", self.hostname, self.port)
        } else {
            self.hostname.clone()
        };

        // Build and return the full URL
        format!("{}://{}{}", protocol, host, self.uri)
    }
}
