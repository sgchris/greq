#![allow(dead_code)] // Disable warnings about dead code globally
#![allow(unused_variables)] // Disable warnings about unused variables globally

use crate::greq_object::greq::Greq;

mod greq_object;
mod cli_parameters;

fn main() {
    let input_file_path = String::from("sample_greqs/sample1.greq");
    let greq_obj = Greq::from_file(&input_file_path);
    if greq_obj.is_err() {
        println!("ERROR: {}", greq_obj.err().unwrap());
        return;
    }

    let greq = greq_obj.unwrap();
    println!("Greq: {}", greq.as_string());

    let results = greq.execute();

    if results.is_err() {
        println!("ERROR: {}", results.err().unwrap());
        return;
    }

    println!("raw response:\r\n{}", results.unwrap());
}
