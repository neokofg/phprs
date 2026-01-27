//! SmartString - Small String Optimization for PHPRS
//!
//! Strings <= 23 bytes are stored inline (no heap allocation).
//! Strings > 23 bytes are stored on the heap.
//!
//! This provides significant performance improvements for typical web workloads
//! where most strings (HTTP headers, JSON keys, short values) are small.

mod ops;

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::{fmt, hash, ptr, slice, str};

pub use ops::*;

/// Maximum length for inline strings (SSO)
const INLINE_CAP: usize = 23;

/// Flag bit in the last byte to indicate heap storage
const HEAP_FLAG: u8 = 0x80;

/// A string type optimized for small strings.
///
/// - Strings up to 23 bytes are stored inline without heap allocation
/// - Larger strings are stored on the heap
/// - Copy-on-write semantics for efficient cloning (TODO)
#[repr(C)]
pub union SmartString {
    inline: InlineString,
    heap: HeapString,
}

/// Inline string storage (24 bytes total)
#[derive(Clone, Copy)]
#[repr(C)]
struct InlineString {
    /// String data (up to 23 bytes)
    data: [u8; INLINE_CAP],
    /// Length in lower 7 bits, high bit = 0 means inline
    len: u8,
}

/// Heap string storage (24 bytes total)
#[derive(Clone, Copy)]
#[repr(C)]
struct HeapString {
    /// Pointer to heap-allocated data
    ptr: *mut u8,
    /// String length
    len: usize,
    /// Allocated capacity (high bit = 1 means heap)
    cap: usize,
}

impl SmartString {
    /// Creates an empty string.
    #[inline]
    pub const fn new() -> Self {
        Self {
            inline: InlineString {
                data: [0; INLINE_CAP],
                len: 0,
            },
        }
    }

    /// Creates a SmartString from a string slice.
    #[inline]
    pub fn from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len();

        if len <= INLINE_CAP {
            // Inline storage
            let mut data = [0u8; INLINE_CAP];
            data[..len].copy_from_slice(bytes);
            Self {
                inline: InlineString {
                    data,
                    len: len as u8,
                },
            }
        } else {
            // Heap storage
            Self::from_str_heap(s)
        }
    }

    /// Creates a heap-allocated string.
    #[cold]
    fn from_str_heap(s: &str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len();
        let cap = len.next_power_of_two().max(32);

        unsafe {
            let layout = Layout::array::<u8>(cap).unwrap();
            let ptr = alloc(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);

            Self {
                heap: HeapString {
                    ptr,
                    len,
                    cap: cap | (HEAP_FLAG as usize) << 56,
                },
            }
        }
    }

    /// Returns `true` if this string is stored inline.
    #[inline]
    pub fn is_inline(&self) -> bool {
        unsafe { self.inline.len & HEAP_FLAG == 0 }
    }

    /// Returns the length of the string in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe {
            if self.is_inline() {
                self.inline.len as usize
            } else {
                self.heap.len
            }
        }
    }

    /// Returns `true` if the string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a byte slice of the string's contents.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            if self.is_inline() {
                &self.inline.data[..self.inline.len as usize]
            } else {
                slice::from_raw_parts(self.heap.ptr, self.heap.len)
            }
        }
    }

    /// Returns a mutable byte slice of the string's contents.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            if self.is_inline() {
                &mut self.inline.data[..self.inline.len as usize]
            } else {
                slice::from_raw_parts_mut(self.heap.ptr, self.heap.len)
            }
        }
    }

    /// Returns the string as a `&str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Returns a raw pointer to the string data.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        unsafe {
            if self.is_inline() {
                self.inline.data.as_ptr()
            } else {
                self.heap.ptr
            }
        }
    }

    /// Appends a string slice to this string.
    pub fn push_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let additional = bytes.len();
        let current_len = self.len();
        let new_len = current_len + additional;

        unsafe {
            if self.is_inline() {
                if new_len <= INLINE_CAP {
                    // Still fits inline
                    ptr::copy_nonoverlapping(
                        bytes.as_ptr(),
                        self.inline.data.as_mut_ptr().add(current_len),
                        additional,
                    );
                    self.inline.len = new_len as u8;
                } else {
                    // Need to move to heap
                    self.grow_to_heap(new_len);
                    ptr::copy_nonoverlapping(
                        bytes.as_ptr(),
                        self.heap.ptr.add(current_len),
                        additional,
                    );
                    self.heap.len = new_len;
                }
            } else {
                // Already on heap
                self.reserve(additional);
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    self.heap.ptr.add(current_len),
                    additional,
                );
                self.heap.len = new_len;
            }
        }
    }

    /// Ensures there's capacity for at least `additional` more bytes.
    fn reserve(&mut self, additional: usize) {
        let current_len = self.len();
        let required = current_len + additional;

        unsafe {
            if self.is_inline() {
                if required > INLINE_CAP {
                    self.grow_to_heap(required);
                }
            } else {
                let current_cap = self.heap.cap & !(0xFF << 56);
                if required > current_cap {
                    let new_cap = required.next_power_of_two();
                    let layout = Layout::array::<u8>(current_cap).unwrap();
                    let new_ptr = realloc(self.heap.ptr, layout, new_cap);
                    if new_ptr.is_null() {
                        std::alloc::handle_alloc_error(Layout::array::<u8>(new_cap).unwrap());
                    }
                    self.heap.ptr = new_ptr;
                    self.heap.cap = new_cap | (HEAP_FLAG as usize) << 56;
                }
            }
        }
    }

    /// Moves an inline string to heap storage.
    #[cold]
    unsafe fn grow_to_heap(&mut self, min_cap: usize) {
        let current_len = self.inline.len as usize;
        let cap = min_cap.next_power_of_two().max(32);

        let layout = Layout::array::<u8>(cap).unwrap();
        let ptr = alloc(layout);
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }

        ptr::copy_nonoverlapping(self.inline.data.as_ptr(), ptr, current_len);

        self.heap = HeapString {
            ptr,
            len: current_len,
            cap: cap | (HEAP_FLAG as usize) << 56,
        };
    }

    /// Concatenates two strings into a new string.
    #[inline]
    pub fn concat(&self, other: &Self) -> Self {
        let len1 = self.len();
        let len2 = other.len();
        let total = len1 + len2;

        if total <= INLINE_CAP {
            // Result fits inline
            let mut data = [0u8; INLINE_CAP];
            data[..len1].copy_from_slice(self.as_bytes());
            data[len1..total].copy_from_slice(other.as_bytes());
            Self {
                inline: InlineString {
                    data,
                    len: total as u8,
                },
            }
        } else {
            // Need heap allocation
            let cap = total.next_power_of_two().max(32);
            unsafe {
                let layout = Layout::array::<u8>(cap).unwrap();
                let ptr = alloc(layout);
                if ptr.is_null() {
                    std::alloc::handle_alloc_error(layout);
                }
                ptr::copy_nonoverlapping(self.as_ptr(), ptr, len1);
                ptr::copy_nonoverlapping(other.as_ptr(), ptr.add(len1), len2);

                Self {
                    heap: HeapString {
                        ptr,
                        len: total,
                        cap: cap | (HEAP_FLAG as usize) << 56,
                    },
                }
            }
        }
    }

    /// Clears the string, making it empty.
    #[inline]
    pub fn clear(&mut self) {
        // Writing to union fields is safe in Rust (only reading requires unsafe)
        if self.is_inline() {
            self.inline.len = 0;
        } else {
            self.heap.len = 0;
        }
    }
}

impl Default for SmartString {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SmartString {
    fn clone(&self) -> Self {
        if self.is_inline() {
            // Inline: just copy the bytes
            Self {
                inline: unsafe { self.inline },
            }
        } else {
            // Heap: allocate new buffer
            Self::from_str(self.as_str())
        }
    }
}

impl Drop for SmartString {
    fn drop(&mut self) {
        unsafe {
            if !self.is_inline() {
                let cap = self.heap.cap & !(0xFF << 56);
                if cap > 0 && !self.heap.ptr.is_null() {
                    let layout = Layout::array::<u8>(cap).unwrap();
                    dealloc(self.heap.ptr, layout);
                }
            }
        }
    }
}

impl fmt::Debug for SmartString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmartString")
            .field("inline", &self.is_inline())
            .field("len", &self.len())
            .field("content", &self.as_str())
            .finish()
    }
}

impl fmt::Display for SmartString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq for SmartString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for SmartString {}

impl PartialEq<str> for SmartString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for SmartString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl hash::Hash for SmartString {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl From<&str> for SmartString {
    #[inline]
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for SmartString {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_str(&s)
    }
}

impl AsRef<str> for SmartString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for SmartString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// =============================================================================
// C ABI exports for use from compiled PHP code
// =============================================================================

/// Create a new SmartString from a raw pointer and length.
#[no_mangle]
pub extern "C" fn rt_string_new(ptr: *const u8, len: usize) -> SmartString {
    if ptr.is_null() || len == 0 {
        return SmartString::new();
    }
    unsafe {
        let slice = slice::from_raw_parts(ptr, len);
        let s = str::from_utf8_unchecked(slice);
        SmartString::from_str(s)
    }
}

/// Get the length of a SmartString.
#[no_mangle]
pub extern "C" fn rt_string_len(s: &SmartString) -> usize {
    s.len()
}

/// Get a pointer to the string data.
#[no_mangle]
pub extern "C" fn rt_string_ptr(s: &SmartString) -> *const u8 {
    s.as_ptr()
}

/// Concatenate two SmartStrings.
#[no_mangle]
pub extern "C" fn rt_string_concat(a: &SmartString, b: &SmartString) -> SmartString {
    a.concat(b)
}

/// Clone a SmartString.
#[no_mangle]
pub extern "C" fn rt_string_clone(s: &SmartString) -> SmartString {
    s.clone()
}

/// Drop a SmartString (free memory if heap-allocated).
#[no_mangle]
pub extern "C" fn rt_string_drop(s: *mut SmartString) {
    if !s.is_null() {
        unsafe {
            ptr::drop_in_place(s);
        }
    }
}

/// Compare two SmartStrings for equality.
#[no_mangle]
pub extern "C" fn rt_string_eq(a: &SmartString, b: &SmartString) -> bool {
    a == b
}

// Убираем unsafe где не нужен
impl SmartString {
    /// Check if two SmartStrings are equal (safe version)
    pub fn equals(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_string() {
        let s = SmartString::from_str("hello");
        assert!(s.is_inline());
        assert_eq!(s.len(), 5);
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_max_inline() {
        let s = SmartString::from_str("12345678901234567890123"); // 23 chars
        assert!(s.is_inline());
        assert_eq!(s.len(), 23);
    }

    #[test]
    fn test_heap_string() {
        let s = SmartString::from_str("123456789012345678901234"); // 24 chars
        assert!(!s.is_inline());
        assert_eq!(s.len(), 24);
        assert_eq!(s.as_str(), "123456789012345678901234");
    }

    #[test]
    fn test_empty_string() {
        let s = SmartString::new();
        assert!(s.is_inline());
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_concat_inline() {
        let a = SmartString::from_str("hello");
        let b = SmartString::from_str(" world");
        let c = a.concat(&b);
        assert!(c.is_inline());
        assert_eq!(c.as_str(), "hello world");
    }

    #[test]
    fn test_concat_to_heap() {
        let a = SmartString::from_str("hello world ");
        let b = SmartString::from_str("this is a longer string");
        let c = a.concat(&b);
        assert!(!c.is_inline());
        assert_eq!(c.as_str(), "hello world this is a longer string");
    }

    #[test]
    fn test_push_str() {
        let mut s = SmartString::from_str("hello");
        s.push_str(" world");
        assert_eq!(s.as_str(), "hello world");
    }

    #[test]
    fn test_push_str_grow_to_heap() {
        let mut s = SmartString::from_str("12345678901234567890"); // 20 chars
        assert!(s.is_inline());
        s.push_str("12345"); // now 25 chars
        assert!(!s.is_inline());
        assert_eq!(s.len(), 25);
    }

    #[test]
    fn test_clone_inline() {
        let a = SmartString::from_str("hello");
        let b = a.clone();
        assert!(b.is_inline());
        assert_eq!(a.as_str(), b.as_str());
    }

    #[test]
    fn test_clone_heap() {
        let a = SmartString::from_str("this is a longer string that goes on heap");
        let b = a.clone();
        assert!(!b.is_inline());
        assert_eq!(a.as_str(), b.as_str());
    }

    #[test]
    fn test_equality() {
        let a = SmartString::from_str("hello");
        let b = SmartString::from_str("hello");
        let c = SmartString::from_str("world");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a, "hello");
    }

    #[test]
    fn test_size() {
        // SmartString should be exactly 24 bytes
        assert_eq!(std::mem::size_of::<SmartString>(), 24);
    }
}
