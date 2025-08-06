use crate::greq_object::greq_footer_condition::ConditionOperator;
use crate::greq_object::greq_response::GreqResponse;
use crate::greq_object::greq_footer_condition::GreqFooterCondition;
use crate::constants::FOOTER_CONDITION_HEADERS_PREFIX;

pub struct GreqEvaluator;

impl GreqEvaluator {

    /// Evaluates a condition against the actual response.
    pub fn evaluate(
        // The actual response
        greq_response: &GreqResponse,
        // the condition to evaluate
        condition: &GreqFooterCondition
    ) -> bool {
        // Check if the condition is a comment
        if condition.is_comment {
            return true;
        }

        // Get the actual value from the response based on the key
        let actual_value = Self::get_response_value(greq_response, &condition.key);

        // Perform the comparison based on the operator
        let comparison_result = Self::compare_values(
            &actual_value,
            &condition.value,
            &condition.operator,
            condition.is_case_sensitive,
        );

        // Apply negation if has_not is true
        if condition.has_not {
            !comparison_result
        } else {
            comparison_result
        }
    }

    /// Extracts the value from the response based on the key.
    pub fn get_response_value(greq_response: &GreqResponse, key: &str) -> String {
        match key {
            "status-code" => greq_response.status_code.to_string(),
            "response-body" => greq_response.body.clone().unwrap_or_default(),
            key if key.starts_with(FOOTER_CONDITION_HEADERS_PREFIX) => {
                if let Some(header_name) = key.strip_prefix(FOOTER_CONDITION_HEADERS_PREFIX) {
                    greq_response.headers.get(header_name).cloned().unwrap_or_default()
                } else {
                    String::new()
                }
            }
            // For direct header access (without "headers." prefix)
            _ => greq_response.headers.get(key).cloned().unwrap_or_default(),
        }
    }

    /// Compares two values based on the specified operator and case sensitivity.
    fn compare_values(
        actual: &str,
        expected: &str,
        operator: &ConditionOperator,
        case_sensitive: bool,
    ) -> bool {
        let (actual_val, expected_val) = if case_sensitive {
            (actual.to_string(), expected.to_string())
        } else {
            (actual.to_lowercase(), expected.to_lowercase())
        };

        match operator {
            ConditionOperator::Equals => actual_val == expected_val,
            ConditionOperator::Contains => actual_val.contains(&expected_val),
            ConditionOperator::StartsWith => actual_val.starts_with(&expected_val),
            ConditionOperator::EndsWith => actual_val.ends_with(&expected_val),
            ConditionOperator::MatchesRegex => {
                if let Ok(regex) = regex::Regex::new(&expected_val) {
                    regex.is_match(&actual_val)
                } else {
                    false
                }
            }
            ConditionOperator::GreaterThan => {
                if let (Ok(actual_num), Ok(expected_num)) = (actual.parse::<f64>(), expected.parse::<f64>()) {
                    actual_num > expected_num
                } else {
                    false
                }
            }
            ConditionOperator::LessThan => {
                if let (Ok(actual_num), Ok(expected_num)) = (actual.parse::<f64>(), expected.parse::<f64>()) {
                    actual_num < expected_num
                } else {
                    false
                }
            }
        }
    }
}
