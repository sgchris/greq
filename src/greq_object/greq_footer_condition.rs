#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Default, Clone, PartialEq)]
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


impl GreqFooterCondition {
    pub fn as_string(&self) -> String {
        format!(
            "{{
    \"is_comment\": {},
    \"key\": \"{}\",
    \"value\": \"{}\",
    \"is_regex\": {},
    \"is_case_sensitive\": {},
    \"operator\": \"{}\",
    \"has_not\": {},
    \"has_or\": {}
}}",
            self.is_comment,
            self.key,
            self.value,
            self.is_regex,
            self.is_case_sensitive,
            self.operator,
            self.has_not,
            self.has_or
        )
    }
}
