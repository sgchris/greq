
pub struct CliTools;

impl CliTools {
    /// Prints a message in green color
    pub fn print_green(message: &str) {
        println!("\x1b[32m{}\x1b[0m", message);
    }

    /// Prints a message in red color
    pub fn print_red(message: &str) {
        println!("\x1b[31m{}\x1b[0m", message);
    }
}

