use crate::greq_object::greq_footer_condition::{ConditionOperator, GreqFooterCondition};
use crate::greq_object::traits::enrich_with_trait::EnrichWith;
use serde::{Deserialize, Serialize};
use crate::constants::{ FOOTER_CONDITION_HEADERS_PREFIX, FOOTER_CONDITION_ALLOWED_KEY_WORDS };
use thiserror::Error;


#[derive(Debug, Error)]
pub enum GreqFooterError {
    /// Occurs when a footer condition line doesn't contain a colon separator
    /// Example: "status-code 200" instead of "status-code: 200"
    #[error("The line does not contain a colon sign")]
    LineHasNoColonSign,
    
    /// Occurs when the "OR" keyword is not at the beginning of a condition
    /// Example: "not or status-code: 200" instead of "or not status-code: 200"
    #[error("The keyword OR is not at the beginning")]
    TheKeywordOrNotInTheBeginning,
    
    /// Occurs when a keyword like "not" appears multiple times in one condition
    /// Example: "not not status-code: 200"
    #[error("The keyword appears more than once")]
    TheKeywordNotAppearsMoreThanOnce,
    
    /// Occurs when a header key format is invalid in headers.* conditions
    /// Example: "headers.: value" or "headers.my.nested.header: value"
    #[error("The header key '{header_key}' is invalid")]
    InvalidHeaderKey { header_key: String },
    
    /// Occurs when an unrecognized keyword is used in a condition
    /// Example: "unknown-keyword: value"
    #[error("The key '{key}' is invalid")]
    InvalidKey { key: String },
}

/// The footer element containing all the test conditions
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GreqFooter {
    pub conditions: Vec<GreqFooterCondition>,
}

impl GreqFooter {
    pub fn parse(footer_lines: &Vec<&str>) -> Result<GreqFooter, GreqFooterError> {
        let mut conditions: Vec<GreqFooterCondition> = Vec::new();

        // loop through each line in the footer
        // Skip empty lines and comments, they aren't relevant for conditions
        for line in footer_lines.iter().map(|l| l.trim()).filter(|l| !l.is_empty()) {

            // Validate and parse each line
            let condition = Self::parse_condition(line)?;
            conditions.push(condition);
        }

        Ok(GreqFooter {
            conditions,
        })
    }

    /// Parse a single condition line into a GreqFooterCondition
    fn parse_condition(line: &str) -> Result<GreqFooterCondition, GreqFooterError> {
        // Split the line on the first ":" to separate key and value
        let (key_part, value_part) = line.split_once(":").unwrap_or_default();
        if key_part.trim().is_empty() || value_part.trim().is_empty() {
            return Err(GreqFooterError::LineHasNoColonSign);
        }

        let mut condition_line: GreqFooterCondition = GreqFooterCondition {
            value: value_part.trim().to_string(),
            ..Default::default()
        };

        // parts consist of "or" "not" "response-body" "matches-regex", etc.
        let key_parts = key_part.split_whitespace()
            .filter(|kp| !kp.is_empty())
            .map(|kp| kp.trim().to_lowercase());

        // the enumeration index for the key_parts
        let mut i: i8 = 0;
        for key in key_parts {

            // Check if the key is a valid operator or prefix
            let is_valid_key = FOOTER_CONDITION_ALLOWED_KEY_WORDS.contains(&key.as_str()) ||
                key.starts_with(FOOTER_CONDITION_HEADERS_PREFIX);
            if !is_valid_key { 
                return Err(GreqFooterError::InvalidKey { key: key });
            }

            // parse the keys
            match key.as_str() {
                // prefixes
                "or" => {
                    if i != 0 {
                        return Err(GreqFooterError::TheKeywordOrNotInTheBeginning);
                    }

                    condition_line.has_or = true;
                }
                "not" => {
                    if condition_line.has_not {
                        return Err(GreqFooterError::TheKeywordNotAppearsMoreThanOnce);
                    }

                    condition_line.has_not = true;
                }

                // the key
                "status-code" => {
                    condition_line.key = "status-code".to_string();
                }
                "response-body" => {
                    condition_line.key = "response-body".to_string();
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
                "matches-regex" => {
                    condition_line.operator = ConditionOperator::MatchesRegex;
                }
                "greater-than" => {
                    condition_line.operator = ConditionOperator::GreaterThan;
                }
                "less-than" => {
                    condition_line.operator = ConditionOperator::LessThan;
                }

                // the suffix
                "case-sensitive" => {
                    condition_line.is_case_sensitive = true;
                }

                // check the headers condition (e.g. "headers.content-type: application/json")
                key if key.starts_with(FOOTER_CONDITION_HEADERS_PREFIX) => {
                    if let Some((_h_prefix, header_name)) = key.split_once(".") {
                        // not allowed to use "." in the header name. E.g. "headers.my.header"
                        if header_name.trim().is_empty() {
                            return Err(GreqFooterError::InvalidHeaderKey { 
                                header_key: key.to_string()
                            });
                        }

                        condition_line.key = header_name.to_string();
                    } else {
                        // should not reach here
                        return Err(GreqFooterError::InvalidHeaderKey {
                            header_key: key.to_string(),
                        });
                    }
                }

                // unknown key used
                _ => {
                    return Err(GreqFooterError::InvalidKey { 
                        key: key.to_string(),
                    })
                }
            }

            i += 1;
        }

        // Create the condition
        Ok(condition_line)
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



