use std::collections::HashMap;

#[derive(Debug)]
struct GreqHeader {
    original_string: String,
    headers: HashMap<String, String>,
}

impl GreqHeader {
    pub fn from_string(contents: &str) -> Result<GreqHeader, &'static str> {
        if !GreqHeader::is_valid(contents) {
            return Err("The provided contents for the header are not valid");
        }
        let mut parsed_headers: HashMap<String, String> = HashMap::new();
        contents.split("\r\n").for_each(|line| {
            let line_parts: Vec<&str> = line.split(":").collect();
            parsed_headers.insert(
                line_parts[0].trim().to_string(),
                line_parts[1].trim().to_string(),
            );
        });

        Ok(GreqHeader {
            original_string: contents.to_string(),
            headers: parsed_headers,
        })
    }

    pub fn is_valid(contents: &str) -> bool {
        // empty contents allowed
        if contents.trim().len() == 0 {
            return true;
        }

        let has_errors: bool = contents.split("\r\n").any(|line| {
            // must not me empty
            if line.len() == 0 {
                return true;
            }

            // must have ":"
            if !line.contains(":") {
                return true;
            }

            // must have at most one ":"
            if line.matches(":").count() > 1 {
                // check that every occurance after the first one, has a "\" prefix
            }

            return false;
        });

        has_errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_test_success() {
        // arrange
        let test_content = "x: 1\r\ny:2";

        // act
        let result: GreqHeader = GreqHeader::from_string(test_content)
            .expect("Could not parse the test contents string");
        println!("greq_header: {:?}", result);

        assert_eq!(result.headers.len(), 2);
        assert!(result.headers.contains_key("x"));
        assert!(result.headers.contains_key("y"));
        assert_eq!(result.headers.get("x").unwrap(), "1");
    }
}
