use greq::logger;
use greq::executor::{execute_multiple_greq_files, print_execution_results, all_successful};
use clap::Parser;
use colored::*;
use std::path::PathBuf;
use std::process;

/// Greq - A robust web API tester with inheritance, dependencies and dynamic requests support
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Greq files to execute
    #[arg(required = true, help = "One or more .greq files to execute")]
    files: Vec<PathBuf>,
    
    /// Enable verbose logging
    #[arg(short, long, help = "Enable verbose logging")]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    // Initialize logger
    if let Err(e) = logger::init_logger() {
        eprintln!("Failed to initialize logger: {e}");
        process::exit(1);
    }
    
    // Set log level based on verbose flag
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }
    
    println!("{}", "Greq - Web API Tester".bold().blue());
    println!("{}", "=".repeat(30).blue());
    
    // Validate files exist
    for file_path in &cli.files {
        if !file_path.exists() {
            eprintln!("{} File not found: {}", "✗".red(), file_path.display());
            process::exit(1);
        }
        
        if file_path.extension().is_none_or(|ext| ext != "greq") {
            eprintln!("{} File must have .greq extension: {}", "⚠".yellow(), file_path.display());
        }
    }
    
    // Execute files
    match execute_multiple_greq_files(&cli.files).await {
        Ok(results) => {
            print_execution_results(&results);
            
            // Exit with appropriate code
            if all_successful(&results) {
                process::exit(0);
            } else {
                process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{} Execution failed: {}", "✗".red(), e);
            process::exit(1);
        }
    }
}
