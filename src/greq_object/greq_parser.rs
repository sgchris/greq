use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::io; 
use std::env;
use fancy_regex::Regex;

use crate::constants::{DELIMITER_MIN_LENGTH, COMMENT_PREFIX};
use crate::greq_object::{
    greq_errors::GreqError,
    greq_response::GreqResponse,
};


/// Parse the first section (header) only. until a generic delimiter is found.
pub fn get_header_section_lines(content: &str) -> Result<Vec<&str>, GreqError> {
    // split the content into lines and trim them
    let lines: Vec<&str> = content.lines()
        .map(|the_line| the_line.trim()) // trim
        .filter(|the_line| !the_line.is_empty()) // remove empty (for the header it's ok)
        .collect();

    // Initialize a vector to hold the header lines
    let mut header_lines: Vec<&str> = Vec::new();

    for line in lines.iter() {
        println!("Parsing line: {}", line); // Debugging line

        // skip comment lines that start with "--"
        // TODO: enable custom delimiter that is non alphanumeric char not a comment
        if line.starts_with(COMMENT_PREFIX) {
            println!("Skipping comment line: {}", line); // Debugging line
            continue; // skip comment lines
        }

        // check if the first char is not a letter or digit, which indicates a delimiter
        let first_char = line.chars().next().unwrap();
        if !first_char.is_alphanumeric() && is_line_only_from_char(line, first_char) {
            println!("Found delimiter line: {}", line); // Debugging line
            break; // stop parsing if the line is a delimiter
        }

        header_lines.push(line);
        println!("Added header line: {}", line); // Debugging line
    }

    Ok(header_lines)
}

// Parse the content of a GREQ file into sections based on a delimiter.
pub fn parse_sections(content: &str, delimiter: char) -> Result<[Vec<&str>; 3], GreqError> {
    let lines: Vec<&str> = content.lines().collect();
    let delimiter_start = delimiter.to_string().repeat(DELIMITER_MIN_LENGTH);

    let mut sections: [Vec<&str>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut part_number = 0usize;

    // itertate over the trimmed lines and split them into sections
    // skip comment lines that start with "--" or "//"
    for line in lines.iter().map(|l| l.trim()).filter(|l| !l.starts_with(COMMENT_PREFIX)) {
        if line.is_empty() && sections[part_number].is_empty() {
            continue; // skip empty lines in the beginning of a section
        }

        // delimiter line is when it starts with at least 4 times the delimiter character
        // and the line contains only that character
        if line.starts_with(&delimiter_start) && is_line_only_from_char(line, delimiter) {
            part_number += 1;
            if part_number > 2 {
                return Err(GreqError::TooManySections);
            }
        } else {
            sections[part_number].push(line);
        }
    }

    if part_number != 2 {
        return Err(GreqError::TooFewSections { found: part_number + 1 });
    }

    Ok(sections)
}

/// Check that a line contains only from a specified character
/// The line may contain whitespace characters.
#[inline]
pub fn is_line_only_from_char(line: &str, character: char) -> bool {
    if line.is_empty() {
        return false; // empty lines are not considered valid
    }

    line.chars().all(|c| c.is_whitespace() || c == character)
}

/// convert header_lines to COW (changeable on write) strings
#[inline(always)]
pub fn strs_to_cows<'a>(strs: &'a Vec<&'a str>) -> Vec<Cow<'a, str>> {
    strs.iter()
        .map(|&s| Cow::from(s))
        .collect()
}

/// Replace placeholders in the lines with values from 
/// get_var in the GreqResponse object.
pub fn replace_placeholders_in_lines(
    lines: &mut Vec<Cow<str>>,
    greq_response: &GreqResponse,
) {
    // regex that finds "$(variable_name)" in the line, without escaping
    let re = Regex::new(r"(?<!\\)\$\(([^)]+)\)").unwrap();

    // replace the placeholders in the header lines
    for line in lines.iter_mut() {
        if !re.is_match(line).unwrap_or(false) {
            continue; // no placeholders to replace
        }

        // replace the placeholders in the line and change to owned COW
        *line = re.replace_all(line, |caps: &fancy_regex::Captures| {
            let var_name = &caps[1];
            greq_response.get_var(var_name)
        }).into_owned().into();
    }
}

/// Check if a file exists, and return its absolute path.
/// If the file path is relative, it will be resolved against the
/// current working directory or a provided base path.
pub fn resolve_and_check_file_exists(
    file_path: &str, // absolute or relative path to the file
    base_path: Option<&str>, // The base path when relative paths are provided
) -> io::Result<String> {
    let candidate_path = if Path::new(file_path).is_absolute() { // Use Path::new for check
        // If the provided path is already absolute, use it directly
        PathBuf::from(file_path) // Convert &str to PathBuf
    } else {
        // If the provided path is relative, resolve it against the base_path
        // or the current working directory if base_path is None.
        let actual_base = match base_path {
            Some(path_str) => PathBuf::from(path_str), // Convert &str to PathBuf
            None => env::current_dir()?, // Get current working directory, propagate error if any
        };
        actual_base.join(file_path) // Join PathBuf with &str
    };

    // Check if the candidate path exists
    if candidate_path.exists() {
        // If it exists, return its canonicalized absolute path as a String.
        // canonicalize() resolves all symlinks, `.` and `..` components.
        candidate_path.canonicalize()? // This returns Result<PathBuf, io::Error>
            .to_str() // Convert Path to &str (Option<&str>)
            .map(|s| s.to_owned()) // Convert &str to String (Option<String>)
            .ok_or_else(|| { // If None (path is not valid UTF-8), return an error
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Resolved path contains invalid Unicode: {}", candidate_path.display()),
                )
            })
    } else {
        // If the file does not exist at the candidate path, return a NotFound error.
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", candidate_path.display()),
        ))
    }
}

