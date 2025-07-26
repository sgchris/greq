/*!
# Greq - A versatile HTTP client for sending requests and evaluating responses

Greq is a command-line HTTP client that supports request inheritance, dependent requests, 
and response evaluation through custom conditions. It reads request definitions from 
specially formatted files and can execute them with comprehensive validation.

## Features
- HTTP/HTTPS request execution
- Request inheritance via base-request files
- Dependent request chaining
- Response evaluation with custom conditions
- Support for various operators (equals, contains, regex, etc.)
- Response output to files
- Raw HTTP format support

## Usage Examples
greq request.greq
greq --display-request-only request.greq

greq --skip-evaluation request.greq
*/

mod constants;
mod cli;
mod greq_object;

use clap::Parser;
use cli::CliParameters;
use greq_object::greq::Greq;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Parse command line arguments
    let args: CliParameters = CliParameters::parse();
    if let Err(validation_error) = args.validate() {
        println!("{}", validation_error);
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, validation_error));
    }

    for input_file in &args.input_files {
        if let Err(e) = process_input_file(input_file).await {
            println!("Error processing file '{}': {}", input_file, e);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
        }
    }

    Ok(())
}

/// Process a single input file and return a Greq object.
#[inline]
pub async fn process_input_file(input_file: &str) -> Result<Greq, String> {
    // parse the input file and initialize the Greq object
    // TODO: Move that part to another async method to handle multiple files simultaneously
    let (greq_response, greq) = Greq::process(&input_file, None).await.map_err(|e| {
        format!("Failed to process input file '{}': {}", input_file, e)
    })?;

    println!("Processed file: {}", input_file);
    if let Some(unwrapped_response) = greq_response {
        println!("Took {}ms", unwrapped_response.response_milliseconds);
    } else {
        println!("No response received.");
    }

    Ok(greq)
}




