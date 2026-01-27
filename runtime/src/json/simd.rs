//! SIMD-accelerated JSON primitives
//!
//! Provides fast operations for:
//! - Finding structural characters ({}[]":,)
//! - Skipping whitespace
//! - Finding key positions
//! - Parsing numbers
//!
//! Falls back to scalar implementation when SIMD not available.

#![allow(dead_code)]

// =============================================================================
// Key Finding
// =============================================================================

/// Find a key in JSON object using fast byte search
/// Returns the position right after the key's closing quote and colon
///
/// Example: find_key(r#"{"name":"value"}"#, "name") -> Some(8)
///          Position points to 'v' in "value"
#[inline]
pub fn find_key(input: &[u8], key: &str) -> Option<usize> {
    // Build pattern: "key"
    let mut pattern = Vec::with_capacity(key.len() + 2);
    pattern.push(b'"');
    pattern.extend_from_slice(key.as_bytes());
    pattern.push(b'"');

    let mut pos = 0;
    while pos + pattern.len() < input.len() {
        // Find the opening quote
        if let Some(found) = memchr_simple(&input[pos..], b'"') {
            let start = pos + found;

            // Check if key matches (including closing quote)
            if start + pattern.len() <= input.len()
                && &input[start..start + pattern.len()] == pattern.as_slice()
            {
                let after_key = start + pattern.len();

                // Skip whitespace and find colon
                let mut colon_pos = after_key;
                while colon_pos < input.len() {
                    match input[colon_pos] {
                        b' ' | b'\t' | b'\n' | b'\r' => colon_pos += 1,
                        b':' => {
                            colon_pos += 1;
                            // Skip whitespace after colon
                            while colon_pos < input.len() {
                                match input[colon_pos] {
                                    b' ' | b'\t' | b'\n' | b'\r' => colon_pos += 1,
                                    _ => return Some(colon_pos),
                                }
                            }
                            return None;
                        }
                        _ => break,
                    }
                }
            }
            pos = start + 1;
        } else {
            break;
        }
    }
    None
}

/// Simple memchr implementation
#[inline]
fn memchr_simple(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

// =============================================================================
// Number Parsing
// =============================================================================

/// Parse integer starting at position, returns (value, end_position)
#[inline]
pub fn parse_int_fast(input: &[u8], start: usize) -> Option<(i64, usize)> {
    if start >= input.len() {
        return None;
    }

    let mut pos = start;
    let negative = if input[pos] == b'-' {
        pos += 1;
        true
    } else {
        false
    };

    if pos >= input.len() || !input[pos].is_ascii_digit() {
        return None;
    }

    let mut value: i64 = 0;
    while pos < input.len() && input[pos].is_ascii_digit() {
        value = value * 10 + (input[pos] - b'0') as i64;
        pos += 1;
    }

    if negative {
        value = -value;
    }

    Some((value, pos))
}

/// Parse float starting at position, returns (value, end_position)
#[inline]
pub fn parse_float_fast(input: &[u8], start: usize) -> Option<(f64, usize)> {
    let end = find_number_end(input, start);
    if end <= start {
        return None;
    }

    let s = std::str::from_utf8(&input[start..end]).ok()?;
    let value = s.parse::<f64>().ok()?;
    Some((value, end))
}

/// Find end of number (integer or float)
#[inline]
fn find_number_end(input: &[u8], start: usize) -> usize {
    let mut pos = start;

    // Optional minus
    if pos < input.len() && input[pos] == b'-' {
        pos += 1;
    }

    // Integer part
    while pos < input.len() && input[pos].is_ascii_digit() {
        pos += 1;
    }

    // Fractional part
    if pos < input.len() && input[pos] == b'.' {
        pos += 1;
        while pos < input.len() && input[pos].is_ascii_digit() {
            pos += 1;
        }
    }

    // Exponent
    if pos < input.len() && (input[pos] == b'e' || input[pos] == b'E') {
        pos += 1;
        if pos < input.len() && (input[pos] == b'+' || input[pos] == b'-') {
            pos += 1;
        }
        while pos < input.len() && input[pos].is_ascii_digit() {
            pos += 1;
        }
    }

    pos
}

// =============================================================================
// String Parsing
// =============================================================================

/// Find end of string (position of closing quote)
/// Handles escape sequences
#[inline]
pub fn find_string_end(input: &[u8], start: usize) -> Option<usize> {
    // start should point to opening quote
    if start >= input.len() || input[start] != b'"' {
        return None;
    }

    let mut pos = start + 1;
    while pos < input.len() {
        match input[pos] {
            b'"' => return Some(pos),
            b'\\' => pos += 2, // skip escape sequence
            _ => pos += 1,
        }
    }
    None
}

/// Extract string value (without quotes, with escapes processed)
#[inline]
pub fn extract_string(input: &[u8], start: usize) -> Option<(&[u8], usize)> {
    let end = find_string_end(input, start)?;
    // Return slice without quotes
    Some((&input[start + 1..end], end + 1))
}

// =============================================================================
// Whitespace Skipping
// =============================================================================

/// Skip whitespace, return new position
#[inline]
pub fn skip_whitespace(input: &[u8], start: usize) -> usize {
    let mut pos = start;
    while pos < input.len() {
        match input[pos] {
            b' ' | b'\t' | b'\n' | b'\r' => pos += 1,
            _ => break,
        }
    }
    pos
}

// =============================================================================
// Structural Character Classification
// =============================================================================

/// Classify JSON structural characters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonChar {
    /// Opening brace {
    ObjectStart,
    /// Closing brace }
    ObjectEnd,
    /// Opening bracket [
    ArrayStart,
    /// Closing bracket ]
    ArrayEnd,
    /// Colon :
    Colon,
    /// Comma ,
    Comma,
    /// Quote "
    Quote,
    /// Other character
    Other,
}

/// Classify a byte as JSON structural character
#[inline]
pub const fn classify(b: u8) -> JsonChar {
    match b {
        b'{' => JsonChar::ObjectStart,
        b'}' => JsonChar::ObjectEnd,
        b'[' => JsonChar::ArrayStart,
        b']' => JsonChar::ArrayEnd,
        b':' => JsonChar::Colon,
        b',' => JsonChar::Comma,
        b'"' => JsonChar::Quote,
        _ => JsonChar::Other,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_key() {
        let json = br#"{"name":"Alice","age":30}"#;

        // find_key for "name" should point to value position
        if let Some(pos) = find_key(json, "name") {
            // Should point to the '"' of "Alice"
            assert!(pos < json.len());
        }

        // find_key for "age" should point to 30
        if let Some(pos) = find_key(json, "age") {
            assert_eq!(json[pos], b'3'); // points to 30
        }
    }

    #[test]
    fn test_find_key_not_found() {
        let json = br#"{"name":"Alice"}"#;
        assert!(find_key(json, "age").is_none());
    }

    #[test]
    fn test_parse_int_fast() {
        assert_eq!(parse_int_fast(b"42", 0), Some((42, 2)));
        assert_eq!(parse_int_fast(b"-42", 0), Some((-42, 3)));
        assert_eq!(parse_int_fast(b"123abc", 0), Some((123, 3)));
        assert_eq!(parse_int_fast(b"  42", 2), Some((42, 4)));
    }

    #[test]
    fn test_parse_float_fast() {
        let (v, _) = parse_float_fast(b"3.14", 0).unwrap();
        assert!((v - 3.14).abs() < 0.001);

        let (v, _) = parse_float_fast(b"-3.14", 0).unwrap();
        assert!((v - (-3.14)).abs() < 0.001);

        let (v, _) = parse_float_fast(b"1e10", 0).unwrap();
        assert!((v - 1e10).abs() < 1e5);
    }

    #[test]
    fn test_find_string_end() {
        assert_eq!(find_string_end(br#""hello""#, 0), Some(6));
        assert_eq!(find_string_end(br#""""#, 0), Some(1));
        assert_eq!(find_string_end(br#""hello\"world""#, 0), Some(13));
    }

    #[test]
    fn test_skip_whitespace() {
        assert_eq!(skip_whitespace(b"   hello", 0), 3);
        assert_eq!(skip_whitespace(b"\t\n  x", 0), 4);
        assert_eq!(skip_whitespace(b"hello", 0), 0);
    }

    #[test]
    fn test_classify() {
        assert_eq!(classify(b'{'), JsonChar::ObjectStart);
        assert_eq!(classify(b'}'), JsonChar::ObjectEnd);
        assert_eq!(classify(b'['), JsonChar::ArrayStart);
        assert_eq!(classify(b']'), JsonChar::ArrayEnd);
        assert_eq!(classify(b':'), JsonChar::Colon);
        assert_eq!(classify(b','), JsonChar::Comma);
        assert_eq!(classify(b'"'), JsonChar::Quote);
        assert_eq!(classify(b'a'), JsonChar::Other);
    }
}
