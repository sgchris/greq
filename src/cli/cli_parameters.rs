use clap::Parser;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliParameterError {
    #[error("No input files specified")]
    NoInputFiles,

    #[error("Input file not found: {file}")]
    FileNotFound { file: String },
}

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
            if let Err(_) = fs::metadata(file) {
                return Err(CliParameterError::FileNotFound {
                    file: file.clone(),
                });
            }
        }

        Ok(())
    }
}

