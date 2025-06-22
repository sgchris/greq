#![allow(dead_code)] // Disable warnings about dead code globally
#![allow(unused_variables)] // Disable warnings about unused variables globally

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

    // Initialize Greq from the input file
    // Parse the file and load the base requests
    // TODO: handle multiple input files
    let first_input_file = args.input_files.first().unwrap();
    let greq = Greq::from_file(&first_input_file)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // Only display the `greq` object without executing it
    // Used when base_request is provided and the user wants to see the merged request
    if args.display_request_only {
        let greq_as_json = serde_json::to_string_pretty(&greq).unwrap_or(String::from("{}"));
        println!("Parse result:\r\n{}", greq_as_json);
        return Ok(());
    }

    if args.skip_evaluation {
        // get only the response
        let response = greq.get_response().await;

        if response.is_ok() {
            let response_as_json = serde_json::to_string_pretty(&response).unwrap_or(String::from("{}"));
            println!("Response:\r\n{}", response_as_json);
        } else {
            println!("ERROR: {}", response.unwrap_err());
        }

        return Ok(());
    }

    // execute greq object
    let result = greq.execute().await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;

    let result_as_json = serde_json::to_string_pretty(&result).unwrap_or(String::from("{}"));
    println!("Response:\r\n{}", result_as_json);

    Ok(())
}
