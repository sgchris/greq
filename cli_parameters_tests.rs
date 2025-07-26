use std::fs::File;
use std::env;
use greq::cli::cli_parameters::*;

#[test]
fn test_validate_error_cases() {
    // Test case 1: No input files
    let params_no_files = CliParameters { 
        input_files: vec![],
        ..Default::default()
    };
    let result = params_no_files.validate();
    assert!(matches!(result, Err(CliParameterError::NoInputFiles)));

    // Test case 2: Single non-existent file
    let params_single_missing = CliParameters { 
        input_files: vec!["nonexistent_file.txt".to_string()],
        ..Default::default()
    };
    let result = params_single_missing.validate();
    assert!(matches!(result, Err(CliParameterError::FileNotFound { .. })));

    // Test case 3: Multiple files with one missing (should fail on first missing)
    let params_mixed = CliParameters { 
        input_files: vec!["missing1.txt".to_string(), "missing2.txt".to_string()],
        ..Default::default()
    };
    let result = params_mixed.validate();
    assert!(matches!(result, Err(CliParameterError::FileNotFound { .. })));
}

#[test]
fn test_validate_success_cases() {
    // Create temporary files for testing
    let mut tmp_path1 = env::temp_dir();
    tmp_path1.push("test_file1.txt");
    File::create(&tmp_path1).unwrap();

    let mut tmp_path2 = env::temp_dir();
    tmp_path2.push("test_file2.txt");
    File::create(&tmp_path2).unwrap();

    // Test case 1: Single valid file
    let params_single = CliParameters { 
        input_files: vec![tmp_path1.to_str().unwrap().to_string()],
        ..Default::default()
    };
    assert!(params_single.validate().is_ok());

    // Test case 2: Multiple valid files
    let params_multiple = CliParameters { 
        input_files: vec![
            tmp_path1.to_str().unwrap().to_string(),
            tmp_path2.to_str().unwrap().to_string()
        ],
        ..Default::default()
    };
    assert!(params_multiple.validate().is_ok());

    // Clean up
    std::fs::remove_file(&tmp_path1).unwrap();
    std::fs::remove_file(&tmp_path2).unwrap();
}


