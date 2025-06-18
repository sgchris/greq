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
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, validation_error));
    }

    // Initialize Greq from the input file
    // Parse the file and load the base requests
    let first_input_file = args.input_files.first().unwrap();
    let greq = Greq::from_file(&first_input_file)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e.message))?;

    if args.display_request_only {
        let greq_as_json = serde_json::to_string_pretty(&greq).unwrap_or(String::from("{}"));
        println!("Parse result:\r\n{}", greq_as_json);
        return Ok(());
    }

    if args.skip_evaluation {
        // get only the response
        let response = greq.get_response().await;
        if response.is_ok() {
            let response_obj = response.unwrap();
            if response_obj.headers.contains_key("content-type") && response_obj.headers["content-type"] == "application/json" {
                if let Some(response_body) = response_obj.body {
                    let greq_as_json = serde_json::to_string_pretty(&response_body).unwrap_or(String::from("{}"));
                    println!("{}", greq_as_json);
                }
            } else {
                println!("{}", response_obj.body.unwrap_or("".to_string()));
            }
        } else {
            println!("ERROR: {}", response.unwrap_err());
        }

        return Ok(());
    }

    // execute greq object
    let results = greq.execute().await;
    if results.is_err() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, results.err().unwrap().message));
    }

    let evaluation_success = results.unwrap().0;
    println!("Succeeded: {:?}", evaluation_success.unwrap());

    Ok(())
}
