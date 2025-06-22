use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ConditionOperator {
    Equals,
    Contains,
    StartsWith,
    EndsWith,

    MatchesRegex,

    // for numerical comparisons
    GreaterThan,
    LessThan,
}

impl Default for ConditionOperator {
    fn default() -> Self { ConditionOperator::Equals }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct GreqFooterCondition {
    // the original line from the footer
    pub original_line: String,

    // commented out line in the condition
    pub is_comment: bool,

    // the response property to check
    // e.g. "status-code", "response-body", "header.content-length", etc.
    pub key: String, 

    // the expected value / regex
    pub value: String,

    // comparison type
    pub is_case_sensitive: bool,

    // the operator to use for comparison. E.g. "equals", "contains", "starts-with", etc.
    pub operator: ConditionOperator, // "equals", "contains", "starts-with", etc.

    // whether this condition is negated
    pub has_not: bool,

    // whether this condition is part of an OR group (to the previous condition)
    pub has_or: bool,
}
