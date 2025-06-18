
use std::fs::File;
use std::env;
use greq::cli::cli_parameters::*;

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

