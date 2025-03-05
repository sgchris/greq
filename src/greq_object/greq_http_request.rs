use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::greq_object::traits::enrich_with_trait::EnrichWith;

// Single request properties
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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

impl EnrichWith for GreqHttpRequest {
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
    where
        Self: Sized,
    {
        // Update method, URI, and content
        self.method = object_to_merge.method.clone();
        self.uri = object_to_merge.uri.clone();
        self.content = object_to_merge.content.clone();
        self.hostname = object_to_merge.hostname.clone();
        self.port = object_to_merge.port;
        self.is_http = object_to_merge.is_http;

        // Merge headers
        for (key, value) in &object_to_merge.headers {
            self.headers.insert(key.clone(), value.clone());
        }

        Ok(())
    }
}
