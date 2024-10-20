use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ConditionOperator {
    Equals,
    Contains,
    StartsWith,
    EndsWith,
}

impl Default for ConditionOperator {
    fn default() -> Self { ConditionOperator::Equals }
}

impl std::fmt::Display for ConditionOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionOperator::Equals => write!(f, "Equals"),
            ConditionOperator::Contains => write!(f, "Contains"),
            ConditionOperator::StartsWith => write!(f, "StartsWith"),
            ConditionOperator::EndsWith => write!(f, "EndsWith"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct GreqFooterCondition {
    pub is_comment: bool,
    pub key: String,
    pub value: String,
    pub is_regex: bool,
    pub is_case_sensitive: bool,
    pub operator: ConditionOperator, // "equals", "contains", "starts-with", etc.
    pub has_not: bool,
    pub has_or: bool,
}
