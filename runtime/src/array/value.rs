//! PHP Value type for PHPRS Runtime
//!
//! Dynamic value type that can hold any PHP value.

use crate::SmartString;

/// PHP value - dynamic type that can hold any PHP value
#[derive(Debug, Clone, PartialEq)]
pub enum PhpValue {
    /// null
    Null,
    /// bool
    Bool(bool),
    /// int (64-bit)
    Int(i64),
    /// float (64-bit)
    Float(f64),
    /// string
    String(SmartString),
    /// array (boxed to avoid infinite size)
    Array(Box<super::PhpArray>),
    // TODO: Object, Resource, Callable
}

impl PhpValue {
    /// Create null value
    #[inline]
    pub const fn null() -> Self {
        PhpValue::Null
    }

    /// Create bool value
    #[inline]
    pub const fn bool(b: bool) -> Self {
        PhpValue::Bool(b)
    }

    /// Create int value
    #[inline]
    pub const fn int(i: i64) -> Self {
        PhpValue::Int(i)
    }

    /// Create float value
    #[inline]
    pub fn float(f: f64) -> Self {
        PhpValue::Float(f)
    }

    /// Create string value
    #[inline]
    pub fn string(s: &str) -> Self {
        PhpValue::String(SmartString::from_str(s))
    }

    /// Create array value
    #[inline]
    pub fn array(arr: super::PhpArray) -> Self {
        PhpValue::Array(Box::new(arr))
    }

    /// Check if value is null
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, PhpValue::Null)
    }

    /// Check if value is truthy (PHP bool conversion)
    pub fn is_truthy(&self) -> bool {
        match self {
            PhpValue::Null => false,
            PhpValue::Bool(b) => *b,
            PhpValue::Int(i) => *i != 0,
            PhpValue::Float(f) => *f != 0.0,
            PhpValue::String(s) => !s.is_empty() && s.as_str() != "0",
            PhpValue::Array(arr) => !arr.is_empty(),
        }
    }

    /// Convert to bool (PHP casting)
    #[inline]
    pub fn to_bool(&self) -> bool {
        self.is_truthy()
    }

    /// Convert to int (PHP casting)
    pub fn to_int(&self) -> i64 {
        match self {
            PhpValue::Null => 0,
            PhpValue::Bool(b) => if *b { 1 } else { 0 },
            PhpValue::Int(i) => *i,
            PhpValue::Float(f) => *f as i64,
            PhpValue::String(s) => s.as_str().parse().unwrap_or(0),
            PhpValue::Array(arr) => if arr.is_empty() { 0 } else { 1 },
        }
    }

    /// Convert to float (PHP casting)
    pub fn to_float(&self) -> f64 {
        match self {
            PhpValue::Null => 0.0,
            PhpValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            PhpValue::Int(i) => *i as f64,
            PhpValue::Float(f) => *f,
            PhpValue::String(s) => s.as_str().parse().unwrap_or(0.0),
            PhpValue::Array(arr) => if arr.is_empty() { 0.0 } else { 1.0 },
        }
    }

    /// Convert to string (PHP casting)
    pub fn to_string(&self) -> SmartString {
        match self {
            PhpValue::Null => SmartString::from_str(""),
            PhpValue::Bool(b) => SmartString::from_str(if *b { "1" } else { "" }),
            PhpValue::Int(i) => SmartString::from_str(&i.to_string()),
            PhpValue::Float(f) => SmartString::from_str(&f.to_string()),
            PhpValue::String(s) => s.clone(),
            PhpValue::Array(_) => SmartString::from_str("Array"),
        }
    }

    /// Get type name
    pub fn type_name(&self) -> &'static str {
        match self {
            PhpValue::Null => "null",
            PhpValue::Bool(_) => "bool",
            PhpValue::Int(_) => "int",
            PhpValue::Float(_) => "float",
            PhpValue::String(_) => "string",
            PhpValue::Array(_) => "array",
        }
    }

    /// Try to get as bool
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PhpValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get as int
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            PhpValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get as float
    #[inline]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            PhpValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Try to get as string reference
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            PhpValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Try to get as array reference
    #[inline]
    pub fn as_array(&self) -> Option<&super::PhpArray> {
        match self {
            PhpValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Try to get as mutable array reference
    #[inline]
    pub fn as_array_mut(&mut self) -> Option<&mut super::PhpArray> {
        match self {
            PhpValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

impl Default for PhpValue {
    fn default() -> Self {
        PhpValue::Null
    }
}

impl From<bool> for PhpValue {
    fn from(b: bool) -> Self {
        PhpValue::Bool(b)
    }
}

impl From<i64> for PhpValue {
    fn from(i: i64) -> Self {
        PhpValue::Int(i)
    }
}

impl From<i32> for PhpValue {
    fn from(i: i32) -> Self {
        PhpValue::Int(i as i64)
    }
}

impl From<f64> for PhpValue {
    fn from(f: f64) -> Self {
        PhpValue::Float(f)
    }
}

impl From<&str> for PhpValue {
    fn from(s: &str) -> Self {
        PhpValue::String(SmartString::from_str(s))
    }
}

impl From<String> for PhpValue {
    fn from(s: String) -> Self {
        PhpValue::String(SmartString::from_str(&s))
    }
}

impl From<SmartString> for PhpValue {
    fn from(s: SmartString) -> Self {
        PhpValue::String(s)
    }
}

// =============================================================================
// C ABI exports
// =============================================================================

/// Create null value
#[no_mangle]
pub extern "C" fn rt_value_null() -> PhpValue {
    PhpValue::Null
}

/// Create bool value
#[no_mangle]
pub extern "C" fn rt_value_bool(b: bool) -> PhpValue {
    PhpValue::Bool(b)
}

/// Create int value
#[no_mangle]
pub extern "C" fn rt_value_int(i: i64) -> PhpValue {
    PhpValue::Int(i)
}

/// Create float value
#[no_mangle]
pub extern "C" fn rt_value_float(f: f64) -> PhpValue {
    PhpValue::Float(f)
}

/// Check if value is truthy
#[no_mangle]
pub extern "C" fn rt_value_is_truthy(value: &PhpValue) -> bool {
    value.is_truthy()
}

/// Convert value to int
#[no_mangle]
pub extern "C" fn rt_value_to_int(value: &PhpValue) -> i64 {
    value.to_int()
}

/// Convert value to float
#[no_mangle]
pub extern "C" fn rt_value_to_float(value: &PhpValue) -> f64 {
    value.to_float()
}

/// Convert value to bool
#[no_mangle]
pub extern "C" fn rt_value_to_bool(value: &PhpValue) -> bool {
    value.to_bool()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truthy() {
        assert!(!PhpValue::Null.is_truthy());
        assert!(!PhpValue::Bool(false).is_truthy());
        assert!(PhpValue::Bool(true).is_truthy());
        assert!(!PhpValue::Int(0).is_truthy());
        assert!(PhpValue::Int(1).is_truthy());
        assert!(PhpValue::Int(-1).is_truthy());
        assert!(!PhpValue::Float(0.0).is_truthy());
        assert!(PhpValue::Float(0.1).is_truthy());
        assert!(!PhpValue::string("").is_truthy());
        assert!(!PhpValue::string("0").is_truthy());
        assert!(PhpValue::string("1").is_truthy());
        assert!(PhpValue::string("hello").is_truthy());
    }

    #[test]
    fn test_to_int() {
        assert_eq!(PhpValue::Null.to_int(), 0);
        assert_eq!(PhpValue::Bool(true).to_int(), 1);
        assert_eq!(PhpValue::Bool(false).to_int(), 0);
        assert_eq!(PhpValue::Int(42).to_int(), 42);
        assert_eq!(PhpValue::Float(3.14).to_int(), 3);
        assert_eq!(PhpValue::string("123").to_int(), 123);
        assert_eq!(PhpValue::string("abc").to_int(), 0);
    }

    #[test]
    fn test_to_float() {
        assert_eq!(PhpValue::Null.to_float(), 0.0);
        assert_eq!(PhpValue::Bool(true).to_float(), 1.0);
        assert_eq!(PhpValue::Int(42).to_float(), 42.0);
        assert_eq!(PhpValue::Float(3.14).to_float(), 3.14);
        assert_eq!(PhpValue::string("3.14").to_float(), 3.14);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(PhpValue::Null.to_string().as_str(), "");
        assert_eq!(PhpValue::Bool(true).to_string().as_str(), "1");
        assert_eq!(PhpValue::Bool(false).to_string().as_str(), "");
        assert_eq!(PhpValue::Int(42).to_string().as_str(), "42");
        assert_eq!(PhpValue::string("hello").to_string().as_str(), "hello");
    }

    #[test]
    fn test_type_name() {
        assert_eq!(PhpValue::Null.type_name(), "null");
        assert_eq!(PhpValue::Bool(true).type_name(), "bool");
        assert_eq!(PhpValue::Int(0).type_name(), "int");
        assert_eq!(PhpValue::Float(0.0).type_name(), "float");
        assert_eq!(PhpValue::string("").type_name(), "string");
    }

    #[test]
    fn test_from_traits() {
        let _: PhpValue = true.into();
        let _: PhpValue = 42i64.into();
        let _: PhpValue = 3.14f64.into();
        let _: PhpValue = "hello".into();
        let _: PhpValue = String::from("world").into();
    }

    #[test]
    fn test_as_methods() {
        let v = PhpValue::Int(42);
        assert_eq!(v.as_int(), Some(42));
        assert_eq!(v.as_bool(), None);
        assert_eq!(v.as_str(), None);
    }
}
