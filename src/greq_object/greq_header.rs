use std::collections::HashMap;

#[derive(Debug)]
struct GreqHeader {
    original_string: String,
    headers: HashMap<String, String>,
}

impl GreqHeader {
    pub fn from_string(contents: &str) -> GreqHeader {
        let mut parsed_headers: HashMap<String, String> = HashMap::new();
        contents.split("\r\n").for_each(|line| {
            let line_parts: Vec<&str> = line.split(":").collect();
            parsed_headers.insert(line_parts[0].to_string(), line_parts[1].to_string());
        });

        GreqHeader {
            original_string: contents.to_string(),
            headers: parsed_headers,
        }
    }
}
