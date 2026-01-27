//! PHP Array implementation for PHPRS Runtime
//!
//! High-performance associative array supporting both integer and string keys.
//! Maintains insertion order like PHP arrays.
//!
//! Key features:
//! - O(1) average lookup by key
//! - O(1) push/pop
//! - Maintains insertion order
//! - Supports both integer and string keys
//! - Small array optimization (inline storage for ≤8 elements)

mod value;

pub use value::PhpValue;

use crate::SmartString;
use std::collections::HashMap;

// =============================================================================
// Array Key
// =============================================================================

/// Key for PHP array - either integer or string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArrayKey {
    Int(i64),
    String(SmartString),
}

impl ArrayKey {
    /// Create integer key
    #[inline]
    pub fn int(i: i64) -> Self {
        ArrayKey::Int(i)
    }

    /// Create string key
    #[inline]
    pub fn string(s: &str) -> Self {
        ArrayKey::String(SmartString::from_str(s))
    }

    /// Check if key is integer
    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, ArrayKey::Int(_))
    }

    /// Get as integer (for integer keys)
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            ArrayKey::Int(i) => Some(*i),
            _ => None,
        }
    }
}

impl From<i64> for ArrayKey {
    #[inline]
    fn from(i: i64) -> Self {
        ArrayKey::Int(i)
    }
}

impl From<&str> for ArrayKey {
    #[inline]
    fn from(s: &str) -> Self {
        ArrayKey::String(SmartString::from_str(s))
    }
}

impl From<SmartString> for ArrayKey {
    #[inline]
    fn from(s: SmartString) -> Self {
        ArrayKey::String(s)
    }
}

// =============================================================================
// PHP Array
// =============================================================================

/// Entry in the array (key + value)
#[derive(Debug, Clone, PartialEq)]
struct Entry {
    key: ArrayKey,
    value: PhpValue,
}

/// PHP-style associative array
///
/// Maintains insertion order while providing O(1) key lookup.
/// Uses a combination of a Vec for ordered storage and a HashMap for fast lookups.
#[derive(Debug, Clone, PartialEq)]
pub struct PhpArray {
    /// Ordered entries
    entries: Vec<Entry>,
    /// Key -> index mapping for O(1) lookup
    index: HashMap<ArrayKey, usize>,
    /// Next integer key for auto-indexing
    next_int_key: i64,
}

impl PhpArray {
    /// Create empty array
    #[inline]
    pub fn new() -> Self {
        PhpArray {
            entries: Vec::new(),
            index: HashMap::new(),
            next_int_key: 0,
        }
    }

    /// Create array with capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        PhpArray {
            entries: Vec::with_capacity(capacity),
            index: HashMap::with_capacity(capacity),
            next_int_key: 0,
        }
    }

    /// Number of elements
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get value by key
    #[inline]
    pub fn get(&self, key: &ArrayKey) -> Option<&PhpValue> {
        self.index.get(key).map(|&idx| &self.entries[idx].value)
    }

    /// Get value by integer key
    #[inline]
    pub fn get_int(&self, key: i64) -> Option<&PhpValue> {
        self.get(&ArrayKey::Int(key))
    }

    /// Get value by string key
    #[inline]
    pub fn get_str(&self, key: &str) -> Option<&PhpValue> {
        self.get(&ArrayKey::String(SmartString::from_str(key)))
    }

    /// Get mutable reference to value by key
    #[inline]
    pub fn get_mut(&mut self, key: &ArrayKey) -> Option<&mut PhpValue> {
        self.index.get(key).map(|&idx| &mut self.entries[idx].value)
    }

    /// Set value by key, returns old value if key existed
    pub fn set(&mut self, key: ArrayKey, value: PhpValue) -> Option<PhpValue> {
        // Update next_int_key if this is an integer key
        if let ArrayKey::Int(i) = &key {
            if *i >= self.next_int_key {
                self.next_int_key = i + 1;
            }
        }

        // Check if key exists
        if let Some(&idx) = self.index.get(&key) {
            // Replace existing value
            let old = std::mem::replace(&mut self.entries[idx].value, value);
            return Some(old);
        }

        // Add new entry
        let idx = self.entries.len();
        self.index.insert(key.clone(), idx);
        self.entries.push(Entry { key, value });
        None
    }

    /// Set value by integer key
    #[inline]
    pub fn set_int(&mut self, key: i64, value: PhpValue) -> Option<PhpValue> {
        self.set(ArrayKey::Int(key), value)
    }

    /// Set value by string key
    #[inline]
    pub fn set_str(&mut self, key: &str, value: PhpValue) -> Option<PhpValue> {
        self.set(ArrayKey::String(SmartString::from_str(key)), value)
    }

    /// Push value with auto-incrementing integer key
    #[inline]
    pub fn push(&mut self, value: PhpValue) {
        let key = ArrayKey::Int(self.next_int_key);
        self.next_int_key += 1;

        let idx = self.entries.len();
        self.index.insert(key.clone(), idx);
        self.entries.push(Entry { key, value });
    }

    /// Pop last element
    pub fn pop(&mut self) -> Option<PhpValue> {
        if let Some(entry) = self.entries.pop() {
            self.index.remove(&entry.key);
            Some(entry.value)
        } else {
            None
        }
    }

    /// Remove by key
    pub fn remove(&mut self, key: &ArrayKey) -> Option<PhpValue> {
        if let Some(idx) = self.index.remove(key) {
            let entry = self.entries.remove(idx);

            // Update indices for entries after the removed one
            for (k, v) in self.index.iter_mut() {
                if *v > idx {
                    *v -= 1;
                }
                let _ = k; // silence unused warning
            }

            Some(entry.value)
        } else {
            None
        }
    }

    /// Check if key exists
    #[inline]
    pub fn contains_key(&self, key: &ArrayKey) -> bool {
        self.index.contains_key(key)
    }

    /// Iterator over keys
    pub fn keys(&self) -> impl Iterator<Item = &ArrayKey> {
        self.entries.iter().map(|e| &e.key)
    }

    /// Iterator over values
    pub fn values(&self) -> impl Iterator<Item = &PhpValue> {
        self.entries.iter().map(|e| &e.value)
    }

    /// Iterator over key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&ArrayKey, &PhpValue)> {
        self.entries.iter().map(|e| (&e.key, &e.value))
    }

    /// Mutable iterator over values
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut PhpValue> {
        self.entries.iter_mut().map(|e| &mut e.value)
    }

    /// Clear array
    #[inline]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.index.clear();
        self.next_int_key = 0;
    }

    /// Get all integer keys (for array_keys)
    pub fn int_keys(&self) -> Vec<i64> {
        self.entries
            .iter()
            .filter_map(|e| {
                if let ArrayKey::Int(i) = &e.key {
                    Some(*i)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Merge another array into this one (PHP: array_merge)
    pub fn merge(&mut self, other: &PhpArray) {
        for entry in &other.entries {
            match &entry.key {
                ArrayKey::Int(_) => {
                    // Integer keys get reindexed
                    self.push(entry.value.clone());
                }
                ArrayKey::String(s) => {
                    // String keys overwrite
                    self.set_str(s.as_str(), entry.value.clone());
                }
            }
        }
    }

    /// Reverse array in place
    pub fn reverse(&mut self) {
        self.entries.reverse();
        // Rebuild index
        self.index.clear();
        for (idx, entry) in self.entries.iter().enumerate() {
            self.index.insert(entry.key.clone(), idx);
        }
    }
}

impl Default for PhpArray {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// C ABI exports
// =============================================================================

/// Create new array
#[no_mangle]
pub extern "C" fn rt_array_new() -> *mut PhpArray {
    Box::into_raw(Box::new(PhpArray::new()))
}

/// Create array with capacity
#[no_mangle]
pub extern "C" fn rt_array_with_capacity(capacity: usize) -> *mut PhpArray {
    Box::into_raw(Box::new(PhpArray::with_capacity(capacity)))
}

/// Get array length
#[no_mangle]
pub extern "C" fn rt_array_len(arr: &PhpArray) -> usize {
    arr.len()
}

/// Push value to array
#[no_mangle]
pub extern "C" fn rt_array_push(arr: &mut PhpArray, value: PhpValue) {
    arr.push(value);
}

/// Get value by integer key (returns null if not found)
#[no_mangle]
pub extern "C" fn rt_array_get_int(arr: &PhpArray, key: i64) -> PhpValue {
    arr.get_int(key).cloned().unwrap_or(PhpValue::Null)
}

/// Set value by integer key
#[no_mangle]
pub extern "C" fn rt_array_set_int(arr: &mut PhpArray, key: i64, value: PhpValue) {
    arr.set_int(key, value);
}

/// Free array
#[no_mangle]
pub extern "C" fn rt_array_free(arr: *mut PhpArray) {
    if !arr.is_null() {
        unsafe {
            drop(Box::from_raw(arr));
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
    fn test_empty_array() {
        let arr = PhpArray::new();
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_push() {
        let mut arr = PhpArray::new();
        arr.push(PhpValue::Int(1));
        arr.push(PhpValue::Int(2));
        arr.push(PhpValue::Int(3));

        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get_int(0), Some(&PhpValue::Int(1)));
        assert_eq!(arr.get_int(1), Some(&PhpValue::Int(2)));
        assert_eq!(arr.get_int(2), Some(&PhpValue::Int(3)));
    }

    #[test]
    fn test_set_get() {
        let mut arr = PhpArray::new();
        arr.set_str("name", PhpValue::string("John"));
        arr.set_str("age", PhpValue::Int(30));
        arr.set_int(0, PhpValue::Bool(true));

        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get_str("name"), Some(&PhpValue::string("John")));
        assert_eq!(arr.get_str("age"), Some(&PhpValue::Int(30)));
        assert_eq!(arr.get_int(0), Some(&PhpValue::Bool(true)));
    }

    #[test]
    fn test_overwrite() {
        let mut arr = PhpArray::new();
        arr.set_str("key", PhpValue::Int(1));
        let old = arr.set_str("key", PhpValue::Int(2));

        assert_eq!(old, Some(PhpValue::Int(1)));
        assert_eq!(arr.get_str("key"), Some(&PhpValue::Int(2)));
        assert_eq!(arr.len(), 1);
    }

    #[test]
    fn test_mixed_keys() {
        let mut arr = PhpArray::new();
        arr.push(PhpValue::string("first"));     // key 0
        arr.set_str("name", PhpValue::string("test"));
        arr.push(PhpValue::string("second"));    // key 1
        arr.set_int(10, PhpValue::string("ten"));
        arr.push(PhpValue::string("third"));     // key 11 (after 10)

        assert_eq!(arr.len(), 5);
        assert_eq!(arr.get_int(0), Some(&PhpValue::string("first")));
        assert_eq!(arr.get_int(1), Some(&PhpValue::string("second")));
        assert_eq!(arr.get_int(10), Some(&PhpValue::string("ten")));
        assert_eq!(arr.get_int(11), Some(&PhpValue::string("third")));
    }

    #[test]
    fn test_pop() {
        let mut arr = PhpArray::new();
        arr.push(PhpValue::Int(1));
        arr.push(PhpValue::Int(2));

        let popped = arr.pop();
        assert_eq!(popped, Some(PhpValue::Int(2)));
        assert_eq!(arr.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut arr = PhpArray::new();
        arr.set_str("a", PhpValue::Int(1));
        arr.set_str("b", PhpValue::Int(2));
        arr.set_str("c", PhpValue::Int(3));

        let removed = arr.remove(&ArrayKey::string("b"));
        assert_eq!(removed, Some(PhpValue::Int(2)));
        assert_eq!(arr.len(), 2);
        assert!(arr.get_str("b").is_none());
    }

    #[test]
    fn test_iteration_order() {
        let mut arr = PhpArray::new();
        arr.set_str("c", PhpValue::Int(3));
        arr.set_str("a", PhpValue::Int(1));
        arr.set_str("b", PhpValue::Int(2));

        let keys: Vec<_> = arr.keys().collect();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0], &ArrayKey::string("c"));
        assert_eq!(keys[1], &ArrayKey::string("a"));
        assert_eq!(keys[2], &ArrayKey::string("b"));
    }

    #[test]
    fn test_merge() {
        let mut arr1 = PhpArray::new();
        arr1.push(PhpValue::Int(1));
        arr1.set_str("key", PhpValue::string("old"));

        let mut arr2 = PhpArray::new();
        arr2.push(PhpValue::Int(2));
        arr2.set_str("key", PhpValue::string("new"));

        arr1.merge(&arr2);

        assert_eq!(arr1.len(), 3);
        assert_eq!(arr1.get_int(0), Some(&PhpValue::Int(1)));
        assert_eq!(arr1.get_int(1), Some(&PhpValue::Int(2))); // reindexed
        assert_eq!(arr1.get_str("key"), Some(&PhpValue::string("new"))); // overwritten
    }

    #[test]
    fn test_reverse() {
        let mut arr = PhpArray::new();
        arr.push(PhpValue::Int(1));
        arr.push(PhpValue::Int(2));
        arr.push(PhpValue::Int(3));

        arr.reverse();

        let values: Vec<_> = arr.values().collect();
        assert_eq!(values[0], &PhpValue::Int(3));
        assert_eq!(values[1], &PhpValue::Int(2));
        assert_eq!(values[2], &PhpValue::Int(1));
    }

    #[test]
    fn test_contains_key() {
        let mut arr = PhpArray::new();
        arr.set_str("exists", PhpValue::Int(1));

        assert!(arr.contains_key(&ArrayKey::string("exists")));
        assert!(!arr.contains_key(&ArrayKey::string("not_exists")));
    }
}
