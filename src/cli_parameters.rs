use clap::Parser;
use std::fs;

#[derive(Parser)]
#[command(name = "Greq")]
#[command(about = "An app to send requests and evaluate responses", long_about = None)]
pub struct CliParameters {
    /// A required input file
    #[arg(short, long)]
    pub input: String,

    // Send request and only show the response, without performing the evaluations introduced in the footer.
    #[arg(short, long)]
    pub request_only: bool,

    // show the result of the parsing process. It is Greq object representation as a JSON.
    #[arg(short, long)]
    pub show_parse_result: bool,
}

impl CliParameters {
    pub fn validate(&self) -> Result<bool, String> {
        if self.input.is_empty() {
            return Err(String::from("Input is empty"));
        }

        match fs::metadata(&self.input) {
            Ok(_) => {},
            Err(_) => return Err(String::from("Input file not found")),
        };

        Ok(true)
    }
}