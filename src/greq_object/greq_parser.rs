
use crate::constants::{DELIMITER_MIN_LENGTH};
use crate::greq_object::greq_errors::GreqError;

// Parse the content of a GREQ file into sections based on a delimiter.
pub fn parse_sections(content: &str, delimiter: char) -> Result<[Vec<&str>; 3], GreqError> {
    let lines: Vec<&str> = content.lines().collect();
    let delimiter_start = delimiter.to_string().repeat(DELIMITER_MIN_LENGTH);
    
    let mut sections: [Vec<&str>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut part_number = 0usize;
    
    // itertate over the trimmed lines and split them into sections
    for line in lines.iter().map(|l| l.trim()) {
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
    line.chars().all(|c| c.is_whitespace() || c == character)
}


// Extract the delimiter character from the content of a GREQ file. Or, use the default delimiter
// if not specified.
pub fn extract_delimiter(content: &str) -> Option<char> {
    content.lines()
        .find(|line| line.to_lowercase().starts_with("delimiter"))
        .and_then(|line| line.split_once(':'))
        .and_then(|(_, value)| value.trim().chars().next())
}

