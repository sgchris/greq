use std::collections::HashMap;

#[derive(Debug)]
struct GreqHeader {
    original_string: String,
    headers: HashMap<String, String>,
}

impl GreqHeader {
    pub fn from_string(contents: &str) -> GreqHeader {
        let greq_header = GreqHeader {
            original_string: contents.to_string(),
            headers: HashMap::new(),
        };

        greq_header
    }
}
