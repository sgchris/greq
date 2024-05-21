#[derive(Debug)]
struct FooterCondition {
    pub key: String,
    pub value: String,
    pub has_or: bool,
    pub has_not: bool,
}

/// The footer element containing all the test conditions
#[derive(Debug)]
struct GreqFooter {
    original_string: String,
    pub conditions: Vec<FooterCondition>,
}
