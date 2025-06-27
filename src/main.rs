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
use greq::cli::cli_tools::CliTools;
use greq_object::greq::Greq;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Parse command line arguments
    let args: CliParameters = CliParameters::parse();
    if let Err(validation_error) = args.validate() {
        println!("{}", validation_error);
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, validation_error));
    }

    // Initialize Greq from the input file
    // Parse the file and load the base requests
    // TODO: handle multiple input files
    let first_input_file = args.input_files.first().unwrap();

    // parse the input file and initialize the Greq object
    // TODO: Move that part to another async method to handle multiple files simultaneously
    let greq_initialization_result = Greq::from_file(&first_input_file);
    if let Err(e) = greq_initialization_result {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
    }
    let greq = greq_initialization_result.unwrap();

    // Only display the `greq` object without executing it
    // Used when base_request is provided and the user wants to see the merged request
    if args.show_request_only {
        let greq_as_json = serde_json::to_string_pretty(&greq).unwrap_or(String::from("{}"));
        println!("Parse result:\r\n{}", greq_as_json);
        return Ok(());
    }

    if args.skip_evaluation {
        // get only the response
        let response = greq.get_response().await;

        if response.is_ok() {
            let response_as_json = serde_json::to_string_pretty(&response).unwrap_or(String::from("{}"));

            if args.show_response {
                // if the user wants to see the response, print it
                println!("Response:\r\n{}", response_as_json);
            }
        } else {
            println!("ERROR: {}", response.unwrap_err());
        }

        return Ok(());
    }

    // execute greq object
    let execution_result_obj = greq.execute(args.show_response).await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    });

    println!();
    if execution_result_obj.is_err() {
        // if the execution failed, print the error
        CliTools::print_red("Failure");
    } else {
        // if the execution was successful, print the success message
        CliTools::print_green("Success");
    }

    Ok(())
}
