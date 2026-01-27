//! Type checking and conversion functions for PHPRS Runtime
//!
//! PHP-style type checking (is_*) and conversion (intval, strval, etc.)
//!
//! These functions work with a tagged union representation where values
//! have a type tag in the high bits of a pointer or are immediate values.

use crate::array::PhpValue;
use crate::SmartString;
use std::ffi::CStr;
use std::os::raw::c_char;

// =============================================================================
// Type Checking via PhpValue
// =============================================================================

/// Check if value is null (PHP: is_null)
#[no_mangle]
pub extern "C" fn rt_is_null(value: &PhpValue) -> bool {
    value.is_null()
}

/// Check if value is int (PHP: is_int/is_integer/is_long)
#[no_mangle]
pub extern "C" fn rt_is_int(value: &PhpValue) -> bool {
    matches!(value, PhpValue::Int(_))
}

/// Check if value is float (PHP: is_float/is_double/is_real)
#[no_mangle]
pub extern "C" fn rt_is_float(value: &PhpValue) -> bool {
    matches!(value, PhpValue::Float(_))
}

/// Check if value is string (PHP: is_string)
#[no_mangle]
pub extern "C" fn rt_is_string(value: &PhpValue) -> bool {
    matches!(value, PhpValue::String(_))
}

/// Check if value is bool (PHP: is_bool)
#[no_mangle]
pub extern "C" fn rt_is_bool(value: &PhpValue) -> bool {
    matches!(value, PhpValue::Bool(_))
}

/// Check if value is array (PHP: is_array)
#[no_mangle]
pub extern "C" fn rt_is_array(value: &PhpValue) -> bool {
    matches!(value, PhpValue::Array(_))
}

/// Check if value is numeric (PHP: is_numeric)
#[no_mangle]
pub extern "C" fn rt_is_numeric(value: &PhpValue) -> bool {
    match value {
        PhpValue::Int(_) | PhpValue::Float(_) => true,
        PhpValue::String(s) => s.as_str().parse::<f64>().is_ok(),
        _ => false,
    }
}

/// Check if value is scalar (PHP: is_scalar)
#[no_mangle]
pub extern "C" fn rt_is_scalar(value: &PhpValue) -> bool {
    matches!(
        value,
        PhpValue::Bool(_) | PhpValue::Int(_) | PhpValue::Float(_) | PhpValue::String(_)
    )
}

// =============================================================================
// Type Conversion via PhpValue
// =============================================================================

/// Convert to int (PHP: intval)
#[no_mangle]
pub extern "C" fn rt_intval(value: &PhpValue) -> i64 {
    value.to_int()
}

/// Convert to float (PHP: floatval/doubleval)
#[no_mangle]
pub extern "C" fn rt_floatval(value: &PhpValue) -> f64 {
    value.to_float()
}

/// Convert to bool (PHP: boolval)
#[no_mangle]
pub extern "C" fn rt_boolval(value: &PhpValue) -> bool {
    value.to_bool()
}

/// Convert to string, returns C string (PHP: strval)
#[no_mangle]
pub extern "C" fn rt_strval(value: &PhpValue) -> *mut c_char {
    let s = value.to_string();
    smart_to_cstr(s)
}

/// Get type name as string (PHP: gettype)
#[no_mangle]
pub extern "C" fn rt_gettype(value: &PhpValue) -> *const c_char {
    match value {
        PhpValue::Null => c"NULL".as_ptr(),
        PhpValue::Bool(_) => c"boolean".as_ptr(),
        PhpValue::Int(_) => c"integer".as_ptr(),
        PhpValue::Float(_) => c"double".as_ptr(),
        PhpValue::String(_) => c"string".as_ptr(),
        PhpValue::Array(_) => c"array".as_ptr(),
    }
}

// =============================================================================
// Simple Type Conversions (for immediate values, not PhpValue)
// =============================================================================

/// Convert int to float
#[no_mangle]
pub extern "C" fn rt_int_to_float(n: i64) -> f64 {
    n as f64
}

/// Convert float to int (truncate)
#[no_mangle]
pub extern "C" fn rt_float_to_int(n: f64) -> i64 {
    n as i64
}

/// Convert bool to int
#[no_mangle]
pub extern "C" fn rt_bool_to_int(b: bool) -> i64 {
    if b {
        1
    } else {
        0
    }
}

/// Convert int to bool
#[no_mangle]
pub extern "C" fn rt_int_to_bool(n: i64) -> bool {
    n != 0
}

/// Convert float to bool
#[no_mangle]
pub extern "C" fn rt_float_to_bool(n: f64) -> bool {
    n != 0.0
}

/// Convert C string to int (PHP: intval for string)
///
/// # Safety
/// `s` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rt_cstr_to_int(s: *const c_char) -> i64 {
    if s.is_null() {
        return 0;
    }
    let cstr = CStr::from_ptr(s);
    cstr.to_str()
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())
        .unwrap_or(0)
}

/// Convert C string to float (PHP: floatval for string)
///
/// # Safety
/// `s` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rt_cstr_to_float(s: *const c_char) -> f64 {
    if s.is_null() {
        return 0.0;
    }
    let cstr = CStr::from_ptr(s);
    cstr.to_str()
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// Convert C string to bool (PHP truthiness for strings)
///
/// # Safety
/// `s` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rt_cstr_to_bool(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }
    let cstr = CStr::from_ptr(s);
    let str = cstr.to_str().unwrap_or("");
    !str.is_empty() && str != "0"
}

/// Convert int to C string
#[no_mangle]
pub extern "C" fn rt_int_to_cstr(n: i64) -> *mut c_char {
    let s = SmartString::from_str(&n.to_string());
    smart_to_cstr(s)
}

/// Convert float to C string
#[no_mangle]
pub extern "C" fn rt_float_to_cstr(n: f64) -> *mut c_char {
    let s = SmartString::from_str(&format_float(n));
    smart_to_cstr(s)
}

/// Convert bool to C string (PHP style: true="1", false="")
#[no_mangle]
pub extern "C" fn rt_bool_to_cstr(b: bool) -> *mut c_char {
    let s = SmartString::from_str(if b { "1" } else { "" });
    smart_to_cstr(s)
}

// =============================================================================
// Empty Check
// =============================================================================

/// Check if value is empty (PHP: empty)
#[no_mangle]
pub extern "C" fn rt_empty(value: &PhpValue) -> bool {
    !value.is_truthy()
}

/// Check if C string is empty
///
/// # Safety
/// `s` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rt_cstr_empty(s: *const c_char) -> bool {
    if s.is_null() {
        return true;
    }
    *s == 0 || (*s == b'0' as i8 && *s.add(1) == 0)
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Format float for display (avoiding unnecessary decimals)
fn format_float(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{:.0}", n)
    } else {
        // Use ryu for fast float formatting
        let mut buf = ryu::Buffer::new();
        buf.format(n).to_string()
    }
}

/// Convert SmartString to C string
fn smart_to_cstr(s: SmartString) -> *mut c_char {
    use std::alloc::{alloc, Layout};

    let bytes = s.as_bytes();
    let len = bytes.len();

    unsafe {
        let layout = Layout::array::<u8>(len + 1).unwrap();
        let ptr = alloc(layout) as *mut c_char;
        if ptr.is_null() {
            return std::ptr::null_mut();
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, ptr, len);
        *ptr.add(len) = 0;
        ptr
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::array::PhpArray;

    #[test]
    fn test_is_null() {
        assert!(rt_is_null(&PhpValue::Null));
        assert!(!rt_is_null(&PhpValue::Int(0)));
    }

    #[test]
    fn test_is_int() {
        assert!(rt_is_int(&PhpValue::Int(42)));
        assert!(!rt_is_int(&PhpValue::Float(42.0)));
    }

    #[test]
    fn test_is_float() {
        assert!(rt_is_float(&PhpValue::Float(3.14)));
        assert!(!rt_is_float(&PhpValue::Int(3)));
    }

    #[test]
    fn test_is_string() {
        assert!(rt_is_string(&PhpValue::string("hello")));
        assert!(!rt_is_string(&PhpValue::Int(0)));
    }

    #[test]
    fn test_is_bool() {
        assert!(rt_is_bool(&PhpValue::Bool(true)));
        assert!(rt_is_bool(&PhpValue::Bool(false)));
        assert!(!rt_is_bool(&PhpValue::Int(1)));
    }

    #[test]
    fn test_is_array() {
        assert!(rt_is_array(&PhpValue::Array(Box::new(PhpArray::new()))));
        assert!(!rt_is_array(&PhpValue::Int(0)));
    }

    #[test]
    fn test_is_numeric() {
        assert!(rt_is_numeric(&PhpValue::Int(42)));
        assert!(rt_is_numeric(&PhpValue::Float(3.14)));
        assert!(rt_is_numeric(&PhpValue::string("123")));
        assert!(rt_is_numeric(&PhpValue::string("3.14")));
        assert!(!rt_is_numeric(&PhpValue::string("hello")));
        assert!(!rt_is_numeric(&PhpValue::Bool(true)));
    }

    #[test]
    fn test_is_scalar() {
        assert!(rt_is_scalar(&PhpValue::Int(1)));
        assert!(rt_is_scalar(&PhpValue::Float(1.0)));
        assert!(rt_is_scalar(&PhpValue::Bool(true)));
        assert!(rt_is_scalar(&PhpValue::string("test")));
        assert!(!rt_is_scalar(&PhpValue::Null));
        assert!(!rt_is_scalar(&PhpValue::Array(Box::new(PhpArray::new()))));
    }

    #[test]
    fn test_intval() {
        assert_eq!(rt_intval(&PhpValue::Int(42)), 42);
        assert_eq!(rt_intval(&PhpValue::Float(3.9)), 3);
        assert_eq!(rt_intval(&PhpValue::Bool(true)), 1);
        assert_eq!(rt_intval(&PhpValue::Bool(false)), 0);
        assert_eq!(rt_intval(&PhpValue::string("123")), 123);
    }

    #[test]
    fn test_floatval() {
        assert_eq!(rt_floatval(&PhpValue::Float(3.14)), 3.14);
        assert_eq!(rt_floatval(&PhpValue::Int(42)), 42.0);
        assert_eq!(rt_floatval(&PhpValue::string("3.14")), 3.14);
    }

    #[test]
    fn test_boolval() {
        assert!(rt_boolval(&PhpValue::Bool(true)));
        assert!(!rt_boolval(&PhpValue::Bool(false)));
        assert!(rt_boolval(&PhpValue::Int(1)));
        assert!(!rt_boolval(&PhpValue::Int(0)));
        assert!(!rt_boolval(&PhpValue::string("")));
        assert!(!rt_boolval(&PhpValue::string("0")));
        assert!(rt_boolval(&PhpValue::string("1")));
    }

    #[test]
    fn test_conversions() {
        assert_eq!(rt_int_to_float(42), 42.0);
        assert_eq!(rt_float_to_int(3.9), 3);
        assert_eq!(rt_bool_to_int(true), 1);
        assert_eq!(rt_bool_to_int(false), 0);
        assert!(rt_int_to_bool(1));
        assert!(!rt_int_to_bool(0));
    }

    #[test]
    fn test_empty() {
        assert!(rt_empty(&PhpValue::Null));
        assert!(rt_empty(&PhpValue::Bool(false)));
        assert!(rt_empty(&PhpValue::Int(0)));
        assert!(rt_empty(&PhpValue::Float(0.0)));
        assert!(rt_empty(&PhpValue::string("")));
        assert!(rt_empty(&PhpValue::string("0")));
        assert!(rt_empty(&PhpValue::Array(Box::new(PhpArray::new()))));

        assert!(!rt_empty(&PhpValue::Bool(true)));
        assert!(!rt_empty(&PhpValue::Int(1)));
        assert!(!rt_empty(&PhpValue::string("hello")));
    }
}
