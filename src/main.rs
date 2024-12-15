#![allow(dead_code)] // Disable warnings about dead code globally
#![allow(unused_variables)] // Disable warnings about unused variables globally

use clap::Parser;
use crate::cli_parameters::CliParameters;
use crate::greq_object::greq::Greq;

mod greq_object;

mod cli_parameters;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = CliParameters::parse();
    if let Err(validation_error) = args.validate() {
        println!("{}", validation_error);
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, validation_error));
    }

    let greq_parse_result = Greq::from_file(&args.input);
    if greq_parse_result.is_err() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, greq_parse_result.err().unwrap().message));
    }

    let greq = greq_parse_result.unwrap();
    if args.show_parse_result {
        let greq_as_json = serde_json::to_string_pretty(&greq).unwrap();
        println!("Parse result:\r\n{}", greq_as_json);
    }

    if args.request_only {
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
