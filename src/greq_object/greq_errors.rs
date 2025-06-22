use thiserror::Error;

// possible Greq parsing errors
#[derive(Debug, PartialEq, Error)]
pub enum GreqError {
    #[error("Too few sections found: expected at least 2 sections (header and content), but found {found}")]
    TooFewSections { found: usize },

    #[error("Too many sections found: expected at most 3 sections (header, content, footer), but found {found}")]
    TooManySections { found: usize },

    #[error("Section separator '{separator}' is not set or invalid")]
    SeparatorNotSet { separator: char },

    #[error("Failed to parse header section: {reason}")]
    ParsingHeaderSectionFailed { reason: String },

    #[error("Failed to parse content section: {reason}")]
    ParsingContentSectionFailed { reason: String },

    #[error("Failed to parse footer section: {reason}")]
    ParsingFooterSectionFailed { reason: String },

    #[error("Failed to read GREQ file '{file_path}'. Reason: {reason}")]
    ReadGreqFileError { file_path: String, reason: String },

    #[error("HTTP request failed: {message}")]
    HttpError { message: String },

    #[error("Failed to enrich request with data from '{dependency}': {reason}")]
    EnrichmentFailed { 
        dependency: String,
        reason: String 
    },

    #[error("Evaluation of condition failed: {reason}")]
    ConditionEvaluationFailed { 
        reason: String 
    },
}


