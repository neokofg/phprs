//! String operations for PHPRS
//!
//! Optimized implementations of common PHP string functions.

use super::SmartString;

// =============================================================================
// Core string functions
// =============================================================================

/// Returns the length of a string (PHP: strlen)
#[no_mangle]
pub extern "C" fn rt_strlen(s: &SmartString) -> i64 {
    s.len() as i64
}

/// Returns a substring (PHP: substr)
///
/// If start is negative, counts from end.
/// If length is negative, stops that many chars from end.
#[no_mangle]
pub extern "C" fn rt_substr(s: &SmartString, start: i64, length: i64) -> SmartString {
    let str_len = s.len() as i64;

    if str_len == 0 {
        return SmartString::new();
    }

    // Handle negative start
    let actual_start = if start < 0 {
        (str_len + start).max(0) as usize
    } else {
        (start as usize).min(s.len())
    };

    // Handle length
    let actual_len = if length < 0 {
        let end = (str_len + length) as usize;
        if end <= actual_start {
            0
        } else {
            end - actual_start
        }
    } else if length == i64::MAX {
        s.len() - actual_start
    } else {
        (length as usize).min(s.len() - actual_start)
    };

    if actual_len == 0 {
        return SmartString::new();
    }

    let bytes = &s.as_bytes()[actual_start..actual_start + actual_len];
    SmartString::from_str(unsafe { std::str::from_utf8_unchecked(bytes) })
}

/// Find the position of a substring (PHP: strpos)
/// Returns -1 if not found.
#[no_mangle]
pub extern "C" fn rt_strpos(haystack: &SmartString, needle: &SmartString) -> i64 {
    if needle.is_empty() {
        return 0;
    }
    if haystack.len() < needle.len() {
        return -1;
    }

    let haystack_bytes = haystack.as_bytes();
    let needle_bytes = needle.as_bytes();

    // Use optimized search
    find_substring(haystack_bytes, needle_bytes)
        .map(|i| i as i64)
        .unwrap_or(-1)
}

/// Find substring using a simple but cache-friendly algorithm
#[inline]
fn find_substring(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.len() == 1 {
        // Single byte search - use memchr
        return memchr_simple(haystack, needle[0]);
    }

    let first = needle[0];
    let mut i = 0;

    while i + needle.len() <= haystack.len() {
        // Find first byte
        if let Some(pos) = memchr_simple(&haystack[i..], first) {
            let start = i + pos;
            if start + needle.len() > haystack.len() {
                return None;
            }
            // Check rest
            if &haystack[start..start + needle.len()] == needle {
                return Some(start);
            }
            i = start + 1;
        } else {
            return None;
        }
    }
    None
}

/// Simple memchr implementation (будет заменено на SIMD)
#[inline]
fn memchr_simple(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

/// Check if string contains substring (PHP: str_contains)
#[no_mangle]
pub extern "C" fn rt_str_contains(haystack: &SmartString, needle: &SmartString) -> bool {
    rt_strpos(haystack, needle) >= 0
}

/// Check if string starts with prefix (PHP: str_starts_with)
#[no_mangle]
pub extern "C" fn rt_str_starts_with(haystack: &SmartString, needle: &SmartString) -> bool {
    haystack.as_bytes().starts_with(needle.as_bytes())
}

/// Check if string ends with suffix (PHP: str_ends_with)
#[no_mangle]
pub extern "C" fn rt_str_ends_with(haystack: &SmartString, needle: &SmartString) -> bool {
    haystack.as_bytes().ends_with(needle.as_bytes())
}

/// Replace occurrences of search with replace (PHP: str_replace)
#[no_mangle]
pub extern "C" fn rt_str_replace(
    search: &SmartString,
    replace: &SmartString,
    subject: &SmartString,
) -> SmartString {
    if search.is_empty() || subject.is_empty() {
        return subject.clone();
    }

    let subject_str = subject.as_str();
    let search_str = search.as_str();
    let replace_str = replace.as_str();

    // Count occurrences first
    let count = subject_str.matches(search_str).count();
    if count == 0 {
        return subject.clone();
    }

    // Build result
    let result = subject_str.replace(search_str, replace_str);
    SmartString::from_str(&result)
}

/// Convert string to lowercase (PHP: strtolower)
#[no_mangle]
pub extern "C" fn rt_strtolower(s: &SmartString) -> SmartString {
    let lower = s.as_str().to_lowercase();
    SmartString::from_str(&lower)
}

/// Convert string to uppercase (PHP: strtoupper)
#[no_mangle]
pub extern "C" fn rt_strtoupper(s: &SmartString) -> SmartString {
    let upper = s.as_str().to_uppercase();
    SmartString::from_str(&upper)
}

/// Trim whitespace from both ends (PHP: trim)
#[no_mangle]
pub extern "C" fn rt_trim(s: &SmartString) -> SmartString {
    SmartString::from_str(s.as_str().trim())
}

/// Trim whitespace from start (PHP: ltrim)
#[no_mangle]
pub extern "C" fn rt_ltrim(s: &SmartString) -> SmartString {
    SmartString::from_str(s.as_str().trim_start())
}

/// Trim whitespace from end (PHP: rtrim)
#[no_mangle]
pub extern "C" fn rt_rtrim(s: &SmartString) -> SmartString {
    SmartString::from_str(s.as_str().trim_end())
}

/// Repeat string n times (PHP: str_repeat)
#[no_mangle]
pub extern "C" fn rt_str_repeat(s: &SmartString, count: i64) -> SmartString {
    if count <= 0 || s.is_empty() {
        return SmartString::new();
    }

    let repeated = s.as_str().repeat(count as usize);
    SmartString::from_str(&repeated)
}

/// Reverse a string (PHP: strrev)
#[no_mangle]
pub extern "C" fn rt_strrev(s: &SmartString) -> SmartString {
    let reversed: String = s.as_str().chars().rev().collect();
    SmartString::from_str(&reversed)
}

/// Pad string to certain length (PHP: str_pad)
/// pad_type: 0 = right, 1 = left, 2 = both
#[no_mangle]
pub extern "C" fn rt_str_pad(
    s: &SmartString,
    length: i64,
    pad_string: &SmartString,
    pad_type: i64,
) -> SmartString {
    let current_len = s.len();
    let target_len = length as usize;

    if target_len <= current_len || pad_string.is_empty() {
        return s.clone();
    }

    let pad_len = target_len - current_len;
    let pad_str = pad_string.as_str();

    let padding: String = pad_str.chars().cycle().take(pad_len).collect();

    let result = match pad_type {
        1 => format!("{}{}", padding, s.as_str()),         // STR_PAD_LEFT
        2 => {                                              // STR_PAD_BOTH
            let left_pad = pad_len / 2;
            let right_pad = pad_len - left_pad;
            let left: String = pad_str.chars().cycle().take(left_pad).collect();
            let right: String = pad_str.chars().cycle().take(right_pad).collect();
            format!("{}{}{}", left, s.as_str(), right)
        }
        _ => format!("{}{}", s.as_str(), padding),         // STR_PAD_RIGHT (default)
    };

    SmartString::from_str(&result)
}

/// Split string by delimiter (PHP: explode)
/// Returns a Vec of SmartStrings (will be converted to PhpArray later)
#[no_mangle]
pub extern "C" fn rt_explode(
    delimiter: &SmartString,
    string: &SmartString,
    limit: i64,
) -> *mut Vec<SmartString> {
    let parts: Vec<SmartString> = if limit <= 0 {
        string
            .as_str()
            .split(delimiter.as_str())
            .map(SmartString::from_str)
            .collect()
    } else {
        string
            .as_str()
            .splitn(limit as usize, delimiter.as_str())
            .map(SmartString::from_str)
            .collect()
    };

    Box::into_raw(Box::new(parts))
}

/// Join array elements with glue (PHP: implode)
/// Note: parts_ptr and parts_len used instead of slice for FFI safety
#[no_mangle]
pub extern "C" fn rt_implode_raw(
    glue: &SmartString,
    parts_ptr: *const SmartString,
    parts_len: usize,
) -> SmartString {
    if parts_ptr.is_null() || parts_len == 0 {
        return SmartString::new();
    }
    let parts = unsafe { std::slice::from_raw_parts(parts_ptr, parts_len) };
    rt_implode(glue, parts)
}

/// Implode with slice (internal, safe Rust)
pub fn rt_implode(glue: &SmartString, parts: &[SmartString]) -> SmartString {
    if parts.is_empty() {
        return SmartString::new();
    }

    let glue_str = glue.as_str();
    let total_len: usize = parts.iter().map(|p| p.len()).sum::<usize>()
        + glue.len() * (parts.len() - 1);

    let mut result = String::with_capacity(total_len);
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            result.push_str(glue_str);
        }
        result.push_str(part.as_str());
    }

    SmartString::from_str(&result)
}

/// Get character at position (PHP: $s[$i] or substr($s, $i, 1))
#[no_mangle]
pub extern "C" fn rt_char_at(s: &SmartString, index: i64) -> SmartString {
    let len = s.len() as i64;
    let actual_index = if index < 0 {
        (len + index) as usize
    } else {
        index as usize
    };

    if actual_index >= s.len() {
        return SmartString::new();
    }

    let bytes = s.as_bytes();
    SmartString::from_str(unsafe {
        std::str::from_utf8_unchecked(&bytes[actual_index..actual_index + 1])
    })
}

/// Get ASCII code of first character (PHP: ord)
#[no_mangle]
pub extern "C" fn rt_ord(s: &SmartString) -> i64 {
    s.as_bytes().first().copied().unwrap_or(0) as i64
}

/// Get character from ASCII code (PHP: chr)
#[no_mangle]
pub extern "C" fn rt_chr(code: i64) -> SmartString {
    let byte = (code & 0xFF) as u8;
    SmartString::from_str(unsafe { std::str::from_utf8_unchecked(&[byte]) })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strlen() {
        let s = SmartString::from_str("hello");
        assert_eq!(rt_strlen(&s), 5);
    }

    #[test]
    fn test_substr() {
        let s = SmartString::from_str("hello world");

        assert_eq!(rt_substr(&s, 0, 5).as_str(), "hello");
        assert_eq!(rt_substr(&s, 6, i64::MAX).as_str(), "world");
        assert_eq!(rt_substr(&s, -5, i64::MAX).as_str(), "world");
        assert_eq!(rt_substr(&s, 0, -6).as_str(), "hello");
    }

    #[test]
    fn test_strpos() {
        let haystack = SmartString::from_str("hello world");
        let needle = SmartString::from_str("world");
        let not_found = SmartString::from_str("xyz");

        assert_eq!(rt_strpos(&haystack, &needle), 6);
        assert_eq!(rt_strpos(&haystack, &not_found), -1);
    }

    #[test]
    fn test_str_contains() {
        let haystack = SmartString::from_str("hello world");
        let needle = SmartString::from_str("world");

        assert!(rt_str_contains(&haystack, &needle));
    }

    #[test]
    fn test_str_replace() {
        let search = SmartString::from_str("world");
        let replace = SmartString::from_str("PHP");
        let subject = SmartString::from_str("hello world");

        let result = rt_str_replace(&search, &replace, &subject);
        assert_eq!(result.as_str(), "hello PHP");
    }

    #[test]
    fn test_strtolower() {
        let s = SmartString::from_str("Hello World");
        assert_eq!(rt_strtolower(&s).as_str(), "hello world");
    }

    #[test]
    fn test_strtoupper() {
        let s = SmartString::from_str("Hello World");
        assert_eq!(rt_strtoupper(&s).as_str(), "HELLO WORLD");
    }

    #[test]
    fn test_trim() {
        let s = SmartString::from_str("  hello  ");
        assert_eq!(rt_trim(&s).as_str(), "hello");
        assert_eq!(rt_ltrim(&s).as_str(), "hello  ");
        assert_eq!(rt_rtrim(&s).as_str(), "  hello");
    }

    #[test]
    fn test_str_repeat() {
        let s = SmartString::from_str("ab");
        assert_eq!(rt_str_repeat(&s, 3).as_str(), "ababab");
    }

    #[test]
    fn test_strrev() {
        let s = SmartString::from_str("hello");
        assert_eq!(rt_strrev(&s).as_str(), "olleh");
    }

    #[test]
    fn test_ord_chr() {
        let s = SmartString::from_str("A");
        assert_eq!(rt_ord(&s), 65);
        assert_eq!(rt_chr(65).as_str(), "A");
    }
}
