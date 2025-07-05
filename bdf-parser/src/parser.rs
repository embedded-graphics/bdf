use std::iter::Enumerate;

/// Parser error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserError {
    pub(crate) message: String,
    pub(crate) line_number: Option<usize>,
}

impl ParserError {
    pub(crate) fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            line_number: None,
        }
    }

    pub(crate) fn with_line(message: &str, line: &Line<'_>) -> Self {
        Self {
            message: message.to_string(),
            line_number: Some(line.line_number),
        }
    }
}

impl std::error::Error for ParserError {}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.line_number {
            write!(f, "line {line}: ")?;
        }
        f.write_str(&self.message)
    }
}

/// Line in a BDF file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line<'a> {
    /// First word in the line, separated by whitespace.
    pub keyword: &'a str,
    /// The remaining text in the line.
    pub parameters: &'a str,

    /// Line number (starting at 1).
    pub line_number: usize,
}

impl<'a> Line<'a> {
    pub fn parse_integer_parameters<const N: usize>(&self) -> Option<[i32; N]> {
        let parts = self
            .parameters
            .split_ascii_whitespace()
            .map(|s| s.parse::<i32>().ok())
            .collect::<Option<Vec<_>>>()?;

        parts.try_into().ok()
    }
}

/// Iterator over lines in a BDF file.
///
/// This iterator keeps track of line numbers for error messages and filters out
/// empty lines and comments.
#[derive(Debug)]
pub struct Lines<'a> {
    input: Enumerate<std::str::Lines<'a>>,
    backtrack_next: Option<Line<'a>>,
}

impl<'a> Lines<'a> {
    /// Creates a new lines iterator.
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.lines().enumerate(),
            backtrack_next: None,
        }
    }

    /// Adds a backtracking line.
    ///
    /// The line that is passed to this method will be returned the next time
    /// [`next`] is called.
    pub fn backtrack(&mut self, line: Line<'a>) {
        assert_eq!(self.backtrack_next, None);

        self.backtrack_next = Some(line);
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = Line<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.backtrack_next.take() {
            return Some(line);
        }

        loop {
            let (index, line) = (&mut self.input)
                .map(|(index, line)| (index, line.trim()))
                .find(|(_, line)| !line.is_empty())?;

            let line = if let Some((keyword, rest)) = line.split_once(char::is_whitespace) {
                Line {
                    keyword,
                    parameters: rest.trim(),
                    line_number: index + 1,
                }
            } else {
                Line {
                    keyword: line,
                    parameters: "",
                    line_number: index + 1,
                }
            };

            if line.keyword != "COMMENT" {
                break Some(line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines() {
        let mut lines = Lines::new("TEST args\n TEST2   some  more args\n\n\t\nNO_ARGS\nCOMMENT\nCOMMENT some comment\nAFTER_COMMENT 123");
        assert_eq!(
            lines.next(),
            Some(Line {
                keyword: "TEST",
                parameters: "args",
                line_number: 1,
            })
        );
        assert_eq!(
            lines.next(),
            Some(Line {
                keyword: "TEST2",
                parameters: "some  more args",
                line_number: 2,
            })
        );
        assert_eq!(
            lines.next(),
            Some(Line {
                keyword: "NO_ARGS",
                parameters: "",
                line_number: 5,
            })
        );
        assert_eq!(
            lines.next(),
            Some(Line {
                keyword: "AFTER_COMMENT",
                parameters: "123",
                line_number: 8,
            })
        );
        assert_eq!(lines.next(), None);
    }
}
