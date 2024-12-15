use std::str::FromStr;
use crate::greq_object::greq_footer_condition::{ConditionOperator, GreqFooterCondition};
use crate::greq_object::traits::enrich_with_trait::EnrichWith;
use serde::{Deserialize, Serialize};

/// The footer element containing all the test conditions
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GreqFooter {
    pub original_string: String,
    pub conditions: Vec<GreqFooterCondition>,
}

#[derive(Debug, PartialEq)]
pub enum GreqFooterErrorCodes {
    LineHasNoColonSign,
    TheKeywordOrNotInTheBeginning,
    TheKeywordNotAppearsMoreThanOnce,
    InvalidHeaderKey,
    InvalidKey
}

#[derive(Debug)]
pub struct GreqFooterError {
    pub code: GreqFooterErrorCodes,
    pub message: String
}

impl GreqFooterErrorCodes {
    pub fn error_message(&self) -> &'static str {
        match self {
            GreqFooterErrorCodes::LineHasNoColonSign => "The line does not contain a colon sign.",
            GreqFooterErrorCodes::TheKeywordOrNotInTheBeginning => "The keyword OR is not at the beginning.",
            GreqFooterErrorCodes::TheKeywordNotAppearsMoreThanOnce => "The keyword appears more than once.",
            GreqFooterErrorCodes::InvalidHeaderKey => "The header key is invalid.",
            GreqFooterErrorCodes::InvalidKey => "The key is invalid.",
        }
    }
}

impl GreqFooterError {
    pub fn new(code: GreqFooterErrorCodes, message: &str) -> Self {
        Self { code, message: message.to_string() }
    }

    pub fn from_error_code(code: GreqFooterErrorCodes) -> Self {
        let error_message = code.error_message();
        Self::new(code, error_message)
    }
}

impl FromStr for GreqFooter {
    type Err = GreqFooterError;

    fn from_str(contents: &str) -> Result<GreqFooter, Self::Err> {
        let mut conditions = Vec::new();
        let original_string = contents.to_string();

        for line in contents.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("--") {
                continue;
            }

            // Validate and parse each line
            let condition = Self::parse_condition(line)?;
            conditions.push(condition);
        }

        Ok(GreqFooter {
            original_string,
            conditions,
        })
    }
}

impl EnrichWith for GreqFooter {
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
    where
        Self: Sized,
    {
        // Merge conditions only if self.conditions is empty
        if self.conditions.is_empty() {
            if !object_to_merge.conditions.is_empty() {
                self.conditions = object_to_merge.conditions.clone(); // Clone the conditions
            }
        } else {
            // add conditions from "object_to_merge" that don't exist in self
            for condition in &object_to_merge.conditions {
                let condition_exists = self
                    .conditions
                    .iter()
                    .find(|existing_self_condition| existing_self_condition.key == condition.key);

                // add the missing footer condition
                if condition_exists.is_none() {
                    self.conditions.push(condition.clone());
                }
            }
        }

        Ok(())
    }
}

impl GreqFooter {
    fn parse_condition(line: &str) -> Result<GreqFooterCondition, GreqFooterError> {
        // Split the line on the first ":" to separate key and value
        let (key_part, value_part) = line.split_once(":").unwrap_or_default();
        if key_part.trim().is_empty() || value_part.is_empty() {
            return Err(GreqFooterError::from_error_code(GreqFooterErrorCodes::LineHasNoColonSign))
        }

        // parts consist of "or" "not" "response-content" "regex", etc.
        let key_parts: Vec<&str> = key_part.split_whitespace().collect();

        let mut condition_line: GreqFooterCondition = GreqFooterCondition {
            value: value_part.trim().to_string(),
            ..Default::default()
        };

        let mut i: i8 = 0;
        key_parts.iter().try_for_each(|key| {
            let lc_key = key.to_lowercase();

            match lc_key.as_str() {
                // prefixes
                "or" => {
                    if i != 0 {
                        return Err(GreqFooterError::from_error_code(GreqFooterErrorCodes::TheKeywordOrNotInTheBeginning));
                    }

                    condition_line.has_or = true;
                }
                "not" => {
                    if condition_line.has_not {
                        return Err(GreqFooterError::from_error_code(GreqFooterErrorCodes::TheKeywordNotAppearsMoreThanOnce));
                    }

                    condition_line.has_not = true;
                }

                // the key
                "status-code" => {
                    condition_line.key = "status-code".to_string();
                }
                "response-content" => {
                    condition_line.key = "response-content".to_string();
                }

                // the operator
                "equals" => {
                    condition_line.operator = ConditionOperator::Equals;
                }
                "contains" => {
                    condition_line.operator = ConditionOperator::Contains;
                }
                "starts-with" => {
                    condition_line.operator = ConditionOperator::StartsWith;
                }
                "ends-with" => {
                    condition_line.operator = ConditionOperator::EndsWith;
                }

                // the suffix
                "regex" => {
                    condition_line.is_regex = true;
                }
                "case-sensitive" => {
                    condition_line.is_case_sensitive = true;
                }

                // check the headers condition (e.g. "headers.content-type: application/json")
                key if key.starts_with("headers.") => {
                    if let Some((h_prefix, header_name)) = key.split_once(".") {
                        // not allowed to use "." in the header name. E.g. "headers.my.header"
                        if header_name.trim().is_empty() || header_name.contains(".") {
                            return Err(GreqFooterError::from_error_code(GreqFooterErrorCodes::InvalidHeaderKey));
                        }

                        condition_line.key = header_name.to_string();
                    } else {
                        // should not reach here
                        return Err(GreqFooterError::from_error_code(GreqFooterErrorCodes::InvalidHeaderKey));
                    }
                }

                // unknown key used
                _ => {
                    return Err(GreqFooterError::from_error_code(GreqFooterErrorCodes::InvalidKey));
                }
            }

            i += 1;
            Ok(())
        })?;

        // Create the condition
        Ok(condition_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_conditions() {
        let input = "status-code equals: 200\nresponse-content contains: \"some content\"\n";
        let footer = GreqFooter::from_str(input).unwrap();
        assert_eq!(footer.conditions.len(), 2);
        assert_eq!(footer.conditions[0].key, "status-code");
        assert_eq!(footer.conditions[0].operator, ConditionOperator::Equals);
        assert_eq!(footer.conditions[0].value, "200");

        assert_eq!(footer.conditions[1].key, "response-content");
        assert_eq!(footer.conditions[1].operator, ConditionOperator::Contains);
        assert_eq!(footer.conditions[1].value, "\"some content\"");
    }

    #[test]
    fn test_ignore_comments() {
        let input = "-- this is a comment\nstatus-code equals: 200\n";
        let footer = GreqFooter::from_str(input).unwrap();
        assert_eq!(footer.conditions.len(), 1);
    }

    #[test]
    fn test_missing_colon() {
        let input = "status-code equals 200\nresponse-content contains: \"some content\"\n";
        let result = GreqFooter::from_str(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, GreqFooterErrorCodes::LineHasNoColonSign);
    }

    #[test]
    fn test_invalid_operator() {
        let input = "status-code invalid-operator: 200\n";
        let result = GreqFooter::from_str(input);
        assert!(result.is_err());

        let result_err = result.unwrap_err();
        assert_eq!(result_err.code, GreqFooterErrorCodes::InvalidKey);
    }

    #[test]
    fn test_conditions_with_prefixes() {
        let input = "not status-code equals: 200\nor response-content contains: \"some content\"\n";
        let parse_result = GreqFooter::from_str(input);
        assert!(!parse_result.is_err());

        let footer = parse_result.unwrap();
        assert_eq!(footer.conditions.len(), 2);
        assert!(footer.conditions[0].has_not);
        assert!(footer.conditions[1].has_or);
    }

    #[test]
    fn test_conditions_with_suffixes() {
        let input = "response-content ends-with case-sensitive: \"the end.\"\n";
        let parse_result = GreqFooter::from_str(input);
        if let Err(err) = &parse_result {
            println!("Parsing error: {}", err.message);
        }

        assert!(!parse_result.is_err());

        let footer = parse_result.unwrap();

        assert_eq!(footer.conditions.len(), 1);
        assert!(footer.conditions[0].is_case_sensitive);
        assert_eq!(footer.conditions[0].value, "\"the end.\"");
    }

    #[test]
    fn test_conditions_with_regex() {
        let input = "response-content contains regex: \"some.*regex\"\n";
        let footer = GreqFooter::from_str(input).unwrap();
        assert_eq!(footer.conditions.len(), 1);
        assert!(footer.conditions[0].is_regex);
        assert_eq!(footer.conditions[0].value, "\"some.*regex\"");
    }

    #[test]
    fn test_multiple_conditions() {
        let input = "status-code equals: 200\nor response-content contains: \"some content\"\nnot response-content starts-with: \"unwanted\"\n";
        let footer = GreqFooter::from_str(input).unwrap();
        assert_eq!(footer.conditions.len(), 3);
        assert_eq!(footer.conditions[2].key, "response-content");
        assert!(footer.conditions[2].has_not);
    }

    #[test]
    fn test_enrich_with_empty_self_non_empty_merge() {
        let mut footer_self = GreqFooter::default();
        let footer_to_merge = GreqFooter {
            conditions: vec![
                GreqFooterCondition {
                    key: "key1".to_string(),
                    value: "value1".to_string(),
                    ..Default::default()
                },
                GreqFooterCondition {
                    key: "key2".to_string(),
                    value: "value2".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        footer_self.enrich_with(&footer_to_merge).unwrap();

        assert_eq!(footer_self.conditions.len(), 2);
        assert_eq!(footer_self.conditions[0].key, "key1");
        assert_eq!(footer_self.conditions[1].key, "key2");
    }

    #[test]
    fn test_enrich_with_non_empty_self_empty_merge() {
        let mut footer_self = GreqFooter {
            conditions: vec![GreqFooterCondition {
                key: "key1".to_string(),
                value: "value1".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let footer_to_merge = GreqFooter::default();

        footer_self.enrich_with(&footer_to_merge).unwrap();

        // self should remain unchanged
        assert_eq!(footer_self.conditions.len(), 1);
        assert_eq!(footer_self.conditions[0].key, "key1");
    }

    #[test]
    fn test_enrich_with_non_empty_self_and_merge_no_duplicates() {
        let mut footer_self = GreqFooter {
            conditions: vec![GreqFooterCondition {
                key: "key1".to_string(),
                value: "value1".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let footer_to_merge = GreqFooter {
            conditions: vec![
                GreqFooterCondition {
                    key: "key1".to_string(), // same key as in self
                    value: "value1".to_string(),
                    ..Default::default()
                },
                GreqFooterCondition {
                    key: "key2".to_string(),
                    value: "value2".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        footer_self.enrich_with(&footer_to_merge).unwrap();

        // Only key2 should be added
        assert_eq!(footer_self.conditions.len(), 2);
        assert_eq!(footer_self.conditions[0].key, "key1");
        assert_eq!(footer_self.conditions[1].key, "key2");
    }

    #[test]
    fn test_enrich_with_duplicates_in_both() {
        let mut footer_self = GreqFooter {
            conditions: vec![
                GreqFooterCondition {
                    key: "key1".to_string(),
                    value: "value1".to_string(),
                    ..Default::default()
                },
                GreqFooterCondition {
                    key: "key2".to_string(),
                    value: "value2".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let footer_to_merge = GreqFooter {
            conditions: vec![
                GreqFooterCondition {
                    key: "key1".to_string(), // duplicate
                    value: "value1".to_string(),
                    ..Default::default()
                },
                GreqFooterCondition {
                    key: "key3".to_string(),
                    value: "value3".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        footer_self.enrich_with(&footer_to_merge).unwrap();

        // key3 should be added, key1 and key2 should remain
        assert_eq!(footer_self.conditions.len(), 3);
        assert_eq!(footer_self.conditions[0].key, "key1");
        assert_eq!(footer_self.conditions[1].key, "key2");
        assert_eq!(footer_self.conditions[2].key, "key3");
    }

    #[test]
    fn test_enrich_with_mixed_conditions() {
        let mut footer_self = GreqFooter {
            conditions: vec![
                GreqFooterCondition {
                    key: "key2".to_string(),
                    value: "value2".to_string(),
                    ..Default::default()
                },
                GreqFooterCondition {
                    key: "key4".to_string(),
                    value: "value4".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let footer_to_merge = GreqFooter {
            conditions: vec![
                GreqFooterCondition {
                    key: "key1".to_string(),
                    value: "value1".to_string(),
                    ..Default::default()
                },
                GreqFooterCondition {
                    key: "key2".to_string(), // duplicate
                    value: "value2".to_string(),
                    ..Default::default()
                },
                GreqFooterCondition {
                    key: "key3".to_string(),
                    value: "value3".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        footer_self.enrich_with(&footer_to_merge).unwrap();

        // key1 and key3 should be added, key2 and key4 remain
        assert_eq!(footer_self.conditions.len(), 4);
        assert_eq!(footer_self.conditions[0].key, "key2");
        assert_eq!(footer_self.conditions[1].key, "key4");
        assert_eq!(footer_self.conditions[2].key, "key1");
        assert_eq!(footer_self.conditions[3].key, "key3");
    }
}
