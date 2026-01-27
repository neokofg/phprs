//! JSON Decoder - High-performance JSON parsing
//!
//! Features:
//! - Zero-copy string parsing where possible
//! - Fast number parsing
//! - Minimal allocations

use super::error::{JsonError, JsonResult};
use crate::array::{PhpArray, PhpValue};
use crate::SmartString;

// =============================================================================
// Decoder
// =============================================================================

/// JSON Decoder with position tracking
pub struct JsonDecoder<'a> {
    /// Input bytes
    input: &'a [u8],
    /// Current position
    pos: usize,
}

impl<'a> JsonDecoder<'a> {
    /// Create new decoder
    #[inline]
    pub fn new(input: &'a str) -> Self {
        JsonDecoder {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    /// Decode the entire input
    pub fn decode(&mut self) -> JsonResult<PhpValue> {
        self.skip_whitespace();
        let value = self.decode_value()?;
        self.skip_whitespace();

        if self.pos < self.input.len() {
            return Err(JsonError::TrailingData(self.pos));
        }

        Ok(value)
    }

    /// Decode a single value
    pub fn decode_value(&mut self) -> JsonResult<PhpValue> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Err(JsonError::UnexpectedEof);
        }

        match self.input[self.pos] {
            b'{' => self.decode_object(),
            b'[' => self.decode_array(),
            b'"' => self.decode_string().map(PhpValue::String),
            b't' => self.decode_true(),
            b'f' => self.decode_false(),
            b'n' => self.decode_null(),
            b'-' | b'0'..=b'9' => self.decode_number(),
            c => Err(JsonError::UnexpectedChar(c as char, self.pos)),
        }
    }

    /// Decode object: { "key": value, ... }
    fn decode_object(&mut self) -> JsonResult<PhpValue> {
        self.pos += 1; // consume '{'
        self.skip_whitespace();

        let mut arr = PhpArray::new();

        // Empty object
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(PhpValue::Array(Box::new(arr)));
        }

        loop {
            self.skip_whitespace();

            // Parse key
            if self.peek() != Some(b'"') {
                return Err(JsonError::ExpectedString(self.pos));
            }
            let key = self.decode_string()?;

            self.skip_whitespace();

            // Expect colon
            if self.peek() != Some(b':') {
                return Err(JsonError::ExpectedColon(self.pos));
            }
            self.pos += 1;

            // Parse value
            let value = self.decode_value()?;
            arr.set_str(key.as_str(), value);

            self.skip_whitespace();

            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    continue;
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(JsonError::ExpectedCommaOrBrace(self.pos)),
            }
        }

        Ok(PhpValue::Array(Box::new(arr)))
    }

    /// Decode array: [ value, ... ]
    fn decode_array(&mut self) -> JsonResult<PhpValue> {
        self.pos += 1; // consume '['
        self.skip_whitespace();

        let mut arr = PhpArray::new();

        // Empty array
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(PhpValue::Array(Box::new(arr)));
        }

        loop {
            let value = self.decode_value()?;
            arr.push(value);

            self.skip_whitespace();

            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    continue;
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(JsonError::ExpectedCommaOrBracket(self.pos)),
            }
        }

        Ok(PhpValue::Array(Box::new(arr)))
    }

    /// Decode string: "..."
    fn decode_string(&mut self) -> JsonResult<SmartString> {
        self.pos += 1; // consume opening '"'

        let start = self.pos;
        let mut has_escapes = false;

        // Fast path: scan for end quote without escapes
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                b'"' => {
                    let s = if has_escapes {
                        self.decode_string_with_escapes(start)?
                    } else {
                        let bytes = &self.input[start..self.pos];
                        SmartString::from_str(unsafe { std::str::from_utf8_unchecked(bytes) })
                    };
                    self.pos += 1; // consume closing '"'
                    return Ok(s);
                }
                b'\\' => {
                    has_escapes = true;
                    self.pos += 2; // skip escape sequence
                }
                _ => self.pos += 1,
            }
        }

        Err(JsonError::UnterminatedString(start))
    }

    /// Decode string with escape sequences
    fn decode_string_with_escapes(&mut self, start: usize) -> JsonResult<SmartString> {
        let mut result = Vec::with_capacity(self.pos - start);
        let mut i = start;

        while i < self.pos {
            if self.input[i] == b'\\' && i + 1 < self.pos {
                i += 1;
                match self.input[i] {
                    b'"' => result.push(b'"'),
                    b'\\' => result.push(b'\\'),
                    b'/' => result.push(b'/'),
                    b'b' => result.push(0x08),
                    b'f' => result.push(0x0C),
                    b'n' => result.push(b'\n'),
                    b'r' => result.push(b'\r'),
                    b't' => result.push(b'\t'),
                    b'u' => {
                        // Unicode escape: \uXXXX
                        if i + 4 < self.pos {
                            let hex = &self.input[i + 1..i + 5];
                            if let Ok(hex_str) = std::str::from_utf8(hex) {
                                if let Ok(code) = u16::from_str_radix(hex_str, 16) {
                                    let c = char::from_u32(code as u32).unwrap_or('\u{FFFD}');
                                    let mut buf = [0u8; 4];
                                    let s = c.encode_utf8(&mut buf);
                                    result.extend_from_slice(s.as_bytes());
                                    i += 4;
                                }
                            }
                        }
                    }
                    c => result.push(c),
                }
            } else {
                result.push(self.input[i]);
            }
            i += 1;
        }

        Ok(SmartString::from_str(unsafe {
            std::str::from_utf8_unchecked(&result)
        }))
    }

    /// Decode number (int or float)
    fn decode_number(&mut self) -> JsonResult<PhpValue> {
        let start = self.pos;
        let mut is_float = false;

        // Optional minus
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }

        // Integer part
        if self.peek() == Some(b'0') {
            self.pos += 1;
        } else {
            while let Some(b'0'..=b'9') = self.peek() {
                self.pos += 1;
            }
        }

        // Fractional part
        if self.peek() == Some(b'.') {
            is_float = true;
            self.pos += 1;
            while let Some(b'0'..=b'9') = self.peek() {
                self.pos += 1;
            }
        }

        // Exponent
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.pos += 1;
            }
            while let Some(b'0'..=b'9') = self.peek() {
                self.pos += 1;
            }
        }

        let s = unsafe { std::str::from_utf8_unchecked(&self.input[start..self.pos]) };

        if is_float {
            s.parse::<f64>()
                .map(PhpValue::Float)
                .map_err(|_| JsonError::InvalidNumber(start))
        } else {
            s.parse::<i64>()
                .map(PhpValue::Int)
                .map_err(|_| JsonError::InvalidNumber(start))
        }
    }

    /// Decode true
    fn decode_true(&mut self) -> JsonResult<PhpValue> {
        if self.input[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(PhpValue::Bool(true))
        } else {
            Err(JsonError::InvalidLiteral(self.pos))
        }
    }

    /// Decode false
    fn decode_false(&mut self) -> JsonResult<PhpValue> {
        if self.input[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(PhpValue::Bool(false))
        } else {
            Err(JsonError::InvalidLiteral(self.pos))
        }
    }

    /// Decode null
    fn decode_null(&mut self) -> JsonResult<PhpValue> {
        if self.input[self.pos..].starts_with(b"null") {
            self.pos += 4;
            Ok(PhpValue::Null)
        } else {
            Err(JsonError::InvalidLiteral(self.pos))
        }
    }

    /// Skip whitespace
    #[inline]
    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    /// Peek current byte
    #[inline]
    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Decode JSON string to PhpValue
pub fn decode(input: &str) -> JsonResult<PhpValue> {
    let mut decoder = JsonDecoder::new(input);
    decoder.decode()
}

/// Decode JSON and return value (alias)
pub fn decode_value(input: &str) -> JsonResult<PhpValue> {
    decode(input)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_int() {
        assert_eq!(decode("42").unwrap(), PhpValue::Int(42));
        assert_eq!(decode("-42").unwrap(), PhpValue::Int(-42));
        assert_eq!(decode("0").unwrap(), PhpValue::Int(0));
    }

    #[test]
    fn test_decode_float() {
        assert_eq!(decode("3.14").unwrap(), PhpValue::Float(3.14));
        assert_eq!(decode("-3.14").unwrap(), PhpValue::Float(-3.14));
        assert_eq!(decode("1e10").unwrap(), PhpValue::Float(1e10));
        assert_eq!(decode("1.5e-3").unwrap(), PhpValue::Float(1.5e-3));
    }

    #[test]
    fn test_decode_bool() {
        assert_eq!(decode("true").unwrap(), PhpValue::Bool(true));
        assert_eq!(decode("false").unwrap(), PhpValue::Bool(false));
    }

    #[test]
    fn test_decode_null() {
        assert_eq!(decode("null").unwrap(), PhpValue::Null);
    }

    #[test]
    fn test_decode_string() {
        assert_eq!(decode(r#""hello""#).unwrap(), PhpValue::string("hello"));
        assert_eq!(decode(r#""""#).unwrap(), PhpValue::string(""));
        assert_eq!(
            decode(r#""hello\nworld""#).unwrap(),
            PhpValue::string("hello\nworld")
        );
    }

    #[test]
    fn test_decode_array() {
        let result = decode("[1, 2, 3]").unwrap();
        if let PhpValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr.get_int(0), Some(&PhpValue::Int(1)));
            assert_eq!(arr.get_int(1), Some(&PhpValue::Int(2)));
            assert_eq!(arr.get_int(2), Some(&PhpValue::Int(3)));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_decode_object() {
        let result = decode(r#"{"a": 1, "b": "test"}"#).unwrap();
        if let PhpValue::Array(arr) = result {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr.get_str("a"), Some(&PhpValue::Int(1)));
            assert_eq!(arr.get_str("b"), Some(&PhpValue::string("test")));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_decode_nested() {
        let result = decode(r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#).unwrap();
        if let PhpValue::Array(arr) = result {
            assert_eq!(arr.len(), 1);
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_decode_whitespace() {
        let result = decode("  {  \"a\"  :  1  }  ").unwrap();
        if let PhpValue::Array(arr) = result {
            assert_eq!(arr.get_str("a"), Some(&PhpValue::Int(1)));
        } else {
            panic!("Expected object");
        }
    }
}
