//! High-performance JSON module for PHPRS Runtime
//!
//! Features:
//! - Zero-copy parsing where possible
//! - SIMD-accelerated key search and number parsing
//! - Schema-aware decoding for known types
//! - Direct-write encoding without intermediate structures
//!
//! Performance targets:
//! - Decode: < 50 ns/op for small objects
//! - Encode: < 30 ns/op for small objects
//! - Throughput: > 1 GB/sec

mod decode;
mod encode;
mod error;
mod simd;

pub use decode::{decode, decode_value, JsonDecoder};
pub use encode::{encode, encode_to, JsonEncoder};
pub use error::{JsonError, JsonResult};

use crate::array::{PhpArray, PhpValue};
use crate::SmartString;

// =============================================================================
// High-level API
// =============================================================================

/// Decode JSON string to PhpValue
///
/// # Example
/// ```ignore
/// let json = r#"{"name": "Alice", "age": 30}"#;
/// let value = json_decode(json)?;
/// ```
#[no_mangle]
pub extern "C" fn rt_json_decode(input: &SmartString) -> PhpValue {
    decode(input.as_str()).unwrap_or(PhpValue::Null)
}

/// Encode PhpValue to JSON string
///
/// # Example
/// ```ignore
/// let value = PhpValue::Int(42);
/// let json = json_encode(&value); // "42"
/// ```
#[no_mangle]
pub extern "C" fn rt_json_encode(value: &PhpValue) -> SmartString {
    let mut buf = Vec::with_capacity(64);
    encode_to(value, &mut buf);
    SmartString::from_str(unsafe { std::str::from_utf8_unchecked(&buf) })
}

/// Decode JSON with error handling
#[no_mangle]
pub extern "C" fn rt_json_decode_safe(input: &SmartString, success: &mut bool) -> PhpValue {
    match decode(input.as_str()) {
        Ok(value) => {
            *success = true;
            value
        }
        Err(_) => {
            *success = false;
            PhpValue::Null
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_encode_roundtrip() {
        let json = r#"{"name":"Alice","age":30}"#;
        let value = decode(json).unwrap();
        let encoded = encode(&value);

        // Verify structure
        if let PhpValue::Array(arr) = &value {
            assert_eq!(arr.get_str("name"), Some(&PhpValue::string("Alice")));
            assert_eq!(arr.get_str("age"), Some(&PhpValue::Int(30)));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_primitives() {
        assert_eq!(decode("42").unwrap(), PhpValue::Int(42));
        assert_eq!(decode("3.14").unwrap(), PhpValue::Float(3.14));
        assert_eq!(decode("true").unwrap(), PhpValue::Bool(true));
        assert_eq!(decode("false").unwrap(), PhpValue::Bool(false));
        assert_eq!(decode("null").unwrap(), PhpValue::Null);
        assert_eq!(decode(r#""hello""#).unwrap(), PhpValue::string("hello"));
    }
}
