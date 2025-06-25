use clap::Parser;
use std::fs;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum CliParameterError {
    NoInputFiles,
    FileNotFound(String, std::io::Error),
}

impl fmt::Display for CliParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliParameterError::NoInputFiles => write!(f, "No input files specified"),
            CliParameterError::FileNotFound(file, err) => write!(f, "Input file not found: {} ({})", file, err),
        }
    }
}

impl Error for CliParameterError {}

#[derive(Parser, Debug, Default)]
#[command(name = "Greq")]
#[command(
    about = "A versatile HTTP client for sending requests and evaluating responses.",
    long_about = "A versatile HTTP client for sending requests and evaluating responses. Features include request inheritance, dependent requests, and support for raw HTTP formats."
)]
pub struct CliParameters {
    // Send request and only show the response, without performing the evaluations introduced in the footer.
    #[arg(short = 's', long)]
    pub skip_evaluation: bool,

    // Display the HTTP response that is going to be received.
    #[arg(short = 'r', long)]
    pub show_response: bool,

    // Display the HTTP request that is going to be sent.
    #[arg(short = 'q', long)]
    pub show_request_only: bool,

    // the greq input files to process
    #[arg()]
    pub input_files: Vec<String>,
}

impl CliParameters {
    pub fn validate(&self) -> Result<(), CliParameterError> {
        if self.input_files.is_empty() {
            return Err(CliParameterError::NoInputFiles);
        }

        for file in &self.input_files {
            if let Err(e) = fs::metadata(file) {
                return Err(CliParameterError::FileNotFound(file.clone(), e));
            }
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::env;

    #[test]
    fn test_validate_no_input_files() {
        let params = CliParameters { 
            input_files: vec![],
            ..Default::default()
        };
        let result = params.validate();
        assert!(matches!(result, Err(CliParameterError::NoInputFiles)));
    }

    #[test]
    fn test_validate_file_not_found() {
        let params = CliParameters { 
            input_files: vec!["nonexistent_file.txt".to_string()],
            ..Default::default()
        };
        let result = params.validate();
        assert!(matches!(result, Err(CliParameterError::FileNotFound(_, _))));
    }

    #[test]
    fn test_validate_success() {
        // Create a temporary file
        let mut tmp_path = env::temp_dir();
        tmp_path.push("test_validate_success.txt");
        File::create(&tmp_path).unwrap();

        let params = CliParameters { 
            input_files: vec![tmp_path.to_str().unwrap().to_string()],
            ..Default::default()
        };
        let result = params.validate();
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_file(&tmp_path).unwrap();
    }
}
