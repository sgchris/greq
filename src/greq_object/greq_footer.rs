use crate::greq_object::greq_footer_condition::{ConditionOperator, GreqFooterCondition};
use crate::greq_object::traits::enrich_with_trait::EnrichWith;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::constants::FOOTER_CONDITION_HEADERS_PREFIX;

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
    InvalidKey,
}

#[derive(Debug)]
pub struct GreqFooterError {
    pub code: GreqFooterErrorCodes,
    pub message: String,
}

impl GreqFooterErrorCodes {
    pub fn error_message(&self) -> &'static str {
        match self {
            GreqFooterErrorCodes::LineHasNoColonSign => "The line does not contain a colon sign.",
            GreqFooterErrorCodes::TheKeywordOrNotInTheBeginning => {
                "The keyword OR is not at the beginning."
            }
            GreqFooterErrorCodes::TheKeywordNotAppearsMoreThanOnce => {
                "The keyword appears more than once."
            }
            GreqFooterErrorCodes::InvalidHeaderKey => "The header key is invalid.",
            GreqFooterErrorCodes::InvalidKey => "The key is invalid.",
        }
    }
}

impl GreqFooterError {
    pub fn new(code: GreqFooterErrorCodes, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
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
            return Err(GreqFooterError::from_error_code(
                GreqFooterErrorCodes::LineHasNoColonSign,
            ));
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
                        return Err(GreqFooterError::from_error_code(
                            GreqFooterErrorCodes::TheKeywordOrNotInTheBeginning,
                        ));
                    }

                    condition_line.has_or = true;
                }
                "not" => {
                    if condition_line.has_not {
                        return Err(GreqFooterError::from_error_code(
                            GreqFooterErrorCodes::TheKeywordNotAppearsMoreThanOnce,
                        ));
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

                // the suffix
                "case-sensitive" => {
                    condition_line.is_case_sensitive = true;
                }

                // check the headers condition (e.g. "headers.content-type: application/json")
                key if key.starts_with(FOOTER_CONDITION_HEADERS_PREFIX) => {
                    if let Some((_h_prefix, header_name)) = key.split_once(".") {
                        // not allowed to use "." in the header name. E.g. "headers.my.header"
                        if header_name.trim().is_empty() || header_name.contains(".") {
                            return Err(GreqFooterError::from_error_code(
                                GreqFooterErrorCodes::InvalidHeaderKey,
                            ));
                        }

                        condition_line.key = header_name.to_string();
                    } else {
                        // should not reach here
                        return Err(GreqFooterError::from_error_code(
                            GreqFooterErrorCodes::InvalidHeaderKey,
                        ));
                    }
                }

                // unknown key used
                _ => {
                    return Err(GreqFooterError::from_error_code(
                        GreqFooterErrorCodes::InvalidKey,
                    ));
                }
            }

            i += 1;
            Ok(())
        })?;

        // Create the condition
        Ok(condition_line)
    }
}

