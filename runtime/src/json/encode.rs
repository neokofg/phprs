//! JSON Encoder - High-performance JSON serialization
//!
//! Features:
//! - Direct-write encoding (no intermediate structures)
//! - Pre-computed field offsets for known types
//! - Minimal allocations

use crate::array::{ArrayKey, PhpArray, PhpValue};
use crate::SmartString;

// =============================================================================
// Encoder
// =============================================================================

/// JSON Encoder
pub struct JsonEncoder {
    /// Output buffer
    buf: Vec<u8>,
}

impl JsonEncoder {
    /// Create new encoder with default capacity
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(256)
    }

    /// Create encoder with specific capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        JsonEncoder {
            buf: Vec::with_capacity(capacity),
        }
    }

    /// Encode a value to JSON
    pub fn encode(&mut self, value: &PhpValue) -> &str {
        self.buf.clear();
        self.write_value(value);
        unsafe { std::str::from_utf8_unchecked(&self.buf) }
    }

    /// Write a value
    fn write_value(&mut self, value: &PhpValue) {
        match value {
            PhpValue::Null => self.write_null(),
            PhpValue::Bool(b) => self.write_bool(*b),
            PhpValue::Int(i) => self.write_int(*i),
            PhpValue::Float(f) => self.write_float(*f),
            PhpValue::String(s) => self.write_string(s.as_str()),
            PhpValue::Array(arr) => self.write_array(arr),
        }
    }

    /// Write null
    #[inline]
    fn write_null(&mut self) {
        self.buf.extend_from_slice(b"null");
    }

    /// Write bool
    #[inline]
    fn write_bool(&mut self, b: bool) {
        if b {
            self.buf.extend_from_slice(b"true");
        } else {
            self.buf.extend_from_slice(b"false");
        }
    }

    /// Write integer - optimized for common cases
    fn write_int(&mut self, i: i64) {
        // Fast path for small positive numbers
        if i >= 0 && i < 100 {
            if i < 10 {
                self.buf.push(b'0' + i as u8);
            } else {
                self.buf.push(b'0' + (i / 10) as u8);
                self.buf.push(b'0' + (i % 10) as u8);
            }
            return;
        }

        // General case - use itoa for speed
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(i);
        self.buf.extend_from_slice(s.as_bytes());
    }

    /// Write float
    fn write_float(&mut self, f: f64) {
        // Handle special cases
        if f.is_nan() || f.is_infinite() {
            self.write_null();
            return;
        }

        // Use ryu for fast float formatting
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format(f);
        self.buf.extend_from_slice(s.as_bytes());
    }

    /// Write string with proper escaping
    fn write_string(&mut self, s: &str) {
        self.buf.push(b'"');

        for byte in s.bytes() {
            match byte {
                b'"' => self.buf.extend_from_slice(b"\\\""),
                b'\\' => self.buf.extend_from_slice(b"\\\\"),
                b'\n' => self.buf.extend_from_slice(b"\\n"),
                b'\r' => self.buf.extend_from_slice(b"\\r"),
                b'\t' => self.buf.extend_from_slice(b"\\t"),
                0x00..=0x1F => {
                    // Control characters: \u00XX
                    self.buf.extend_from_slice(b"\\u00");
                    self.buf.push(HEX_CHARS[(byte >> 4) as usize]);
                    self.buf.push(HEX_CHARS[(byte & 0xF) as usize]);
                }
                _ => self.buf.push(byte),
            }
        }

        self.buf.push(b'"');
    }

    /// Write array or object
    fn write_array(&mut self, arr: &PhpArray) {
        // Determine if this is an array (sequential int keys) or object (string keys)
        let is_sequential = self.is_sequential_array(arr);

        if is_sequential {
            self.write_json_array(arr);
        } else {
            self.write_json_object(arr);
        }
    }

    /// Check if array has sequential integer keys starting from 0
    fn is_sequential_array(&self, arr: &PhpArray) -> bool {
        let mut expected = 0i64;
        for key in arr.keys() {
            match key {
                ArrayKey::Int(i) if *i == expected => expected += 1,
                _ => return false,
            }
        }
        true
    }

    /// Write as JSON array: [...]
    fn write_json_array(&mut self, arr: &PhpArray) {
        self.buf.push(b'[');

        let mut first = true;
        for value in arr.values() {
            if !first {
                self.buf.push(b',');
            }
            first = false;
            self.write_value(value);
        }

        self.buf.push(b']');
    }

    /// Write as JSON object: {...}
    fn write_json_object(&mut self, arr: &PhpArray) {
        self.buf.push(b'{');

        let mut first = true;
        for (key, value) in arr.iter() {
            if !first {
                self.buf.push(b',');
            }
            first = false;

            // Write key
            match key {
                ArrayKey::Int(i) => {
                    self.buf.push(b'"');
                    self.write_int(*i);
                    self.buf.push(b'"');
                }
                ArrayKey::String(s) => {
                    self.write_string(s.as_str());
                }
            }

            self.buf.push(b':');
            self.write_value(value);
        }

        self.buf.push(b'}');
    }

    /// Get the encoded bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

impl Default for JsonEncoder {
    fn default() -> Self {
        Self::new()
    }
}

// Hex characters for escape sequences
const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

// =============================================================================
// Public API
// =============================================================================

/// Encode PhpValue to JSON string
pub fn encode(value: &PhpValue) -> SmartString {
    let mut encoder = JsonEncoder::new();
    let json = encoder.encode(value);
    SmartString::from_str(json)
}

/// Encode PhpValue to existing buffer
pub fn encode_to(value: &PhpValue, buf: &mut Vec<u8>) {
    let mut encoder = JsonEncoder { buf: std::mem::take(buf) };
    encoder.write_value(value);
    *buf = encoder.buf;
}

// =============================================================================
// Fast integer writing (inline for small numbers)
// =============================================================================

/// Write integer directly to buffer (optimized)
#[inline]
pub fn write_int_fast(buf: &mut Vec<u8>, i: i64) {
    let mut buffer = itoa::Buffer::new();
    let s = buffer.format(i);
    buf.extend_from_slice(s.as_bytes());
}

/// Write string with escaping directly to buffer
#[inline]
pub fn write_string_escaped(buf: &mut Vec<u8>, s: &str) {
    buf.push(b'"');
    for byte in s.bytes() {
        match byte {
            b'"' => buf.extend_from_slice(b"\\\""),
            b'\\' => buf.extend_from_slice(b"\\\\"),
            b'\n' => buf.extend_from_slice(b"\\n"),
            b'\r' => buf.extend_from_slice(b"\\r"),
            b'\t' => buf.extend_from_slice(b"\\t"),
            0x00..=0x1F => {
                buf.extend_from_slice(b"\\u00");
                buf.push(HEX_CHARS[(byte >> 4) as usize]);
                buf.push(HEX_CHARS[(byte & 0xF) as usize]);
            }
            _ => buf.push(byte),
        }
    }
    buf.push(b'"');
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_null() {
        assert_eq!(encode(&PhpValue::Null).as_str(), "null");
    }

    #[test]
    fn test_encode_bool() {
        assert_eq!(encode(&PhpValue::Bool(true)).as_str(), "true");
        assert_eq!(encode(&PhpValue::Bool(false)).as_str(), "false");
    }

    #[test]
    fn test_encode_int() {
        assert_eq!(encode(&PhpValue::Int(0)).as_str(), "0");
        assert_eq!(encode(&PhpValue::Int(42)).as_str(), "42");
        assert_eq!(encode(&PhpValue::Int(-42)).as_str(), "-42");
        assert_eq!(encode(&PhpValue::Int(12345678)).as_str(), "12345678");
    }

    #[test]
    fn test_encode_float() {
        assert_eq!(encode(&PhpValue::Float(3.14)).as_str(), "3.14");
        assert_eq!(encode(&PhpValue::Float(-3.14)).as_str(), "-3.14");
    }

    #[test]
    fn test_encode_string() {
        assert_eq!(encode(&PhpValue::string("hello")).as_str(), r#""hello""#);
        assert_eq!(encode(&PhpValue::string("")).as_str(), r#""""#);
        assert_eq!(
            encode(&PhpValue::string("hello\nworld")).as_str(),
            r#""hello\nworld""#
        );
        assert_eq!(
            encode(&PhpValue::string("say \"hi\"")).as_str(),
            r#""say \"hi\"""#
        );
    }

    #[test]
    fn test_encode_array() {
        let mut arr = PhpArray::new();
        arr.push(PhpValue::Int(1));
        arr.push(PhpValue::Int(2));
        arr.push(PhpValue::Int(3));

        assert_eq!(encode(&PhpValue::Array(Box::new(arr))).as_str(), "[1,2,3]");
    }

    #[test]
    fn test_encode_object() {
        let mut arr = PhpArray::new();
        arr.set_str("name", PhpValue::string("Alice"));
        arr.set_str("age", PhpValue::Int(30));

        let json = encode(&PhpValue::Array(Box::new(arr)));
        assert!(json.as_str().contains(r#""name":"Alice""#));
        assert!(json.as_str().contains(r#""age":30"#));
    }

    #[test]
    fn test_encode_nested() {
        let mut inner = PhpArray::new();
        inner.push(PhpValue::Int(1));
        inner.push(PhpValue::Int(2));

        let mut outer = PhpArray::new();
        outer.set_str("data", PhpValue::Array(Box::new(inner)));

        let json = encode(&PhpValue::Array(Box::new(outer)));
        assert_eq!(json.as_str(), r#"{"data":[1,2]}"#);
    }
}
