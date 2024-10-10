use clap::Parser;

#[derive(Parser)]
#[command(name = "Greq")]
#[command(about = "An app to send requests and evaluate responses", long_about = None)]
pub struct CliParameters {
    /// A required input file
    #[arg(short, long)]
    input: String,

    // show the output of the request without evaluations.
    #[arg(short, long)]
    request_only: bool,

}