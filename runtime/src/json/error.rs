//! JSON Error types

use std::fmt;

/// JSON parsing/encoding result
pub type JsonResult<T> = Result<T, JsonError>;

/// JSON error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonError {
    /// Unexpected end of input
    UnexpectedEof,
    /// Unexpected character at position
    UnexpectedChar(char, usize),
    /// Expected string (for object key)
    ExpectedString(usize),
    /// Expected colon after key
    ExpectedColon(usize),
    /// Expected comma or closing brace
    ExpectedCommaOrBrace(usize),
    /// Expected comma or closing bracket
    ExpectedCommaOrBracket(usize),
    /// Unterminated string starting at position
    UnterminatedString(usize),
    /// Invalid number at position
    InvalidNumber(usize),
    /// Invalid literal (true/false/null)
    InvalidLiteral(usize),
    /// Trailing data after JSON value
    TrailingData(usize),
    /// Maximum depth exceeded
    MaxDepthExceeded,
    /// Invalid UTF-8
    InvalidUtf8,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::UnexpectedEof => write!(f, "Unexpected end of JSON input"),
            JsonError::UnexpectedChar(c, pos) => {
                write!(f, "Unexpected character '{}' at position {}", c, pos)
            }
            JsonError::ExpectedString(pos) => {
                write!(f, "Expected string at position {}", pos)
            }
            JsonError::ExpectedColon(pos) => {
                write!(f, "Expected ':' at position {}", pos)
            }
            JsonError::ExpectedCommaOrBrace(pos) => {
                write!(f, "Expected ',' or '}}' at position {}", pos)
            }
            JsonError::ExpectedCommaOrBracket(pos) => {
                write!(f, "Expected ',' or ']' at position {}", pos)
            }
            JsonError::UnterminatedString(pos) => {
                write!(f, "Unterminated string starting at position {}", pos)
            }
            JsonError::InvalidNumber(pos) => {
                write!(f, "Invalid number at position {}", pos)
            }
            JsonError::InvalidLiteral(pos) => {
                write!(f, "Invalid literal at position {}", pos)
            }
            JsonError::TrailingData(pos) => {
                write!(f, "Trailing data after JSON value at position {}", pos)
            }
            JsonError::MaxDepthExceeded => {
                write!(f, "Maximum nesting depth exceeded")
            }
            JsonError::InvalidUtf8 => {
                write!(f, "Invalid UTF-8 in JSON string")
            }
        }
    }
}

impl std::error::Error for JsonError {}
