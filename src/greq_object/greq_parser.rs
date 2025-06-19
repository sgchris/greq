
use crate::constants::{DELIMITER_MIN_LENGTH};
use crate::greq_object::greq::GreqErrorCodes;

// Parse the content of a GREQ file into sections based on a delimiter.
pub fn parse_sections(content: &str, delimiter: char) -> Result<[Vec<&str>; 3], GreqErrorCodes> {
    let lines: Vec<&str> = content.lines().collect();
    let delimiter_start = delimiter.to_string().repeat(DELIMITER_MIN_LENGTH);
    
    let mut sections: [Vec<&str>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut part_number = 0usize;

    for line in lines.iter() {
        if line.starts_with(&delimiter_start) {
            part_number += 1;
            if part_number > 2 {
                return Err(GreqErrorCodes::TooManySections);
            }
        } else {
            sections[part_number].push(*line);
        }
    }

    if part_number != 2 {
        return Err(GreqErrorCodes::TooFewSections);
    }

    Ok(sections)
}


// Extract the delimiter character from the content of a GREQ file. Or, use the default delimiter
// if not specified.
pub fn extract_delimiter(content: &str) -> Option<char> {
    content.lines()
        .find(|line| line.to_lowercase().starts_with("delimiter"))
        .and_then(|line| line.split_once(':'))
        .and_then(|(_, value)| value.trim().chars().next())
}

