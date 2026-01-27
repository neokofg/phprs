//! HTTP Headers with InternedString optimization

use crate::intern::{intern, InternId};

/// Maximum number of headers stored inline (no heap allocation)
const INLINE_HEADERS: usize = 16;

/// HTTP Headers collection
/// Uses InternedString for header names (O(1) comparison)
/// Stores up to 16 headers inline without heap allocation
#[derive(Debug, Clone)]
pub struct Headers<'a> {
    inline: [Option<(InternId, &'a str)>; INLINE_HEADERS],
    inline_len: usize,
    overflow: Option<Vec<(InternId, &'a str)>>,
}

impl<'a> Headers<'a> {
    /// Create empty headers
    #[inline]
    pub const fn new() -> Self {
        Self {
            inline: [None; INLINE_HEADERS],
            inline_len: 0,
            overflow: None,
        }
    }

    /// Get header value by name (case-insensitive)
    #[inline]
    pub fn get(&self, name: &str) -> Option<&'a str> {
        let lower = name.to_ascii_lowercase();
        let id = intern(&lower);
        self.get_by_id(id)
    }

    /// Get header value by interned ID (fastest)
    #[inline]
    pub fn get_by_id(&self, id: InternId) -> Option<&'a str> {
        // Search inline first
        for i in 0..self.inline_len {
            if let Some((header_id, value)) = &self.inline[i] {
                if *header_id == id {
                    return Some(*value);
                }
            }
        }

        // Search overflow if exists
        if let Some(overflow) = &self.overflow {
            for (header_id, value) in overflow {
                if *header_id == id {
                    return Some(*value);
                }
            }
        }

        None
    }

    /// Insert a header (name is stored case-insensitively)
    #[inline]
    pub fn insert(&mut self, name: &str, value: &'a str) {
        let lower = name.to_ascii_lowercase();
        let id = intern(&lower);
        self.insert_by_id(id, value);
    }

    /// Insert a header by interned ID
    #[inline]
    pub fn insert_by_id(&mut self, id: InternId, value: &'a str) {
        // Check if exists and update
        for i in 0..self.inline_len {
            if let Some((header_id, ref mut header_value)) = &mut self.inline[i] {
                if *header_id == id {
                    *header_value = value;
                    return;
                }
            }
        }

        if let Some(overflow) = &mut self.overflow {
            for (header_id, header_value) in overflow.iter_mut() {
                if *header_id == id {
                    *header_value = value;
                    return;
                }
            }
        }

        // Insert new
        if self.inline_len < INLINE_HEADERS {
            self.inline[self.inline_len] = Some((id, value));
            self.inline_len += 1;
        } else {
            self.overflow.get_or_insert_with(Vec::new).push((id, value));
        }
    }

    /// Get number of headers
    #[inline]
    pub fn len(&self) -> usize {
        self.inline_len + self.overflow.as_ref().map_or(0, |v| v.len())
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inline_len == 0
    }

    /// Iterate over headers
    #[inline]
    pub fn iter(&self) -> HeadersIter<'_, 'a> {
        HeadersIter {
            headers: self,
            inline_idx: 0,
            overflow_idx: 0,
        }
    }

    /// Get Content-Length header as usize
    #[inline]
    pub fn content_length(&self) -> Option<usize> {
        self.get("content-length").and_then(|v| v.parse().ok())
    }

    /// Get Content-Type header
    #[inline]
    pub fn content_type(&self) -> Option<&'a str> {
        self.get("content-type")
    }

    /// Check if connection should keep-alive
    #[inline]
    pub fn is_keep_alive(&self) -> bool {
        self.get("connection")
            .is_none_or(|v| !v.eq_ignore_ascii_case("close"))
    }
}

impl Default for Headers<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over headers
pub struct HeadersIter<'h, 'a> {
    headers: &'h Headers<'a>,
    inline_idx: usize,
    overflow_idx: usize,
}

impl<'h, 'a> Iterator for HeadersIter<'h, 'a> {
    type Item = (InternId, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate inline first
        while self.inline_idx < self.headers.inline_len {
            let idx = self.inline_idx;
            self.inline_idx += 1;
            if let Some((id, value)) = self.headers.inline[idx] {
                return Some((id, value));
            }
        }

        // Then overflow
        if let Some(overflow) = &self.headers.overflow {
            if self.overflow_idx < overflow.len() {
                let idx = self.overflow_idx;
                self.overflow_idx += 1;
                return Some(overflow[idx]);
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.headers.len() - self.inline_idx - self.overflow_idx;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for HeadersIter<'_, '_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headers_basic() {
        let mut headers = Headers::new();
        assert!(headers.is_empty());

        headers.insert("Content-Type", "application/json");
        headers.insert("Content-Length", "42");

        assert_eq!(headers.len(), 2);
        assert_eq!(headers.get("content-type"), Some("application/json"));
        assert_eq!(headers.get("content-length"), Some("42"));
        assert_eq!(headers.content_length(), Some(42));
    }

    #[test]
    fn test_headers_update() {
        let mut headers = Headers::new();
        headers.insert("Content-Type", "text/plain");
        headers.insert("Content-Type", "application/json");

        assert_eq!(headers.len(), 1);
        assert_eq!(headers.content_type(), Some("application/json"));
    }

    #[test]
    fn test_headers_overflow() {
        let mut headers = Headers::new();

        // Fill inline storage
        for i in 0..20 {
            let name = format!("X-Header-{}", i);
            // We need static lifetime, so use leaked string for test
            let name: &'static str = Box::leak(name.into_boxed_str());
            headers.insert(name, "value");
        }

        assert_eq!(headers.len(), 20);
    }

    #[test]
    fn test_keep_alive() {
        let mut headers = Headers::new();
        assert!(headers.is_keep_alive()); // default

        headers.insert("Connection", "keep-alive");
        assert!(headers.is_keep_alive());

        headers.insert("Connection", "close");
        assert!(!headers.is_keep_alive());
    }
}
