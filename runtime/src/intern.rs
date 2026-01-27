//! String Interning for PHPRS Runtime
//!
//! Interned strings allow O(1) equality comparison via pointer/id comparison
//! instead of O(n) byte comparison. Perfect for HTTP headers and common strings.
//!
//! Two interning strategies:
//! 1. Static interning - compile-time known strings (HTTP headers)
//! 2. Dynamic interning - runtime string deduplication

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::collections::HashMap;
use std::sync::RwLock;

// =============================================================================
// Static Interned Strings (HTTP Headers)
// =============================================================================

/// Interned string ID - small integer for fast comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct InternId(u32);

impl InternId {
    /// Create from raw value (for FFI)
    #[inline]
    pub const fn from_raw(id: u32) -> Self {
        InternId(id)
    }

    /// Get raw value
    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Unknown/invalid ID
    pub const UNKNOWN: InternId = InternId(0);
}

// =============================================================================
// Common HTTP Headers (compile-time interned)
// =============================================================================

/// Pre-interned HTTP header names
/// These are the most common headers, interned at compile time
#[allow(non_upper_case_globals)]
pub mod headers {
    use super::InternId;

    // Request headers
    pub const Accept: InternId = InternId(1);
    pub const AcceptCharset: InternId = InternId(2);
    pub const AcceptEncoding: InternId = InternId(3);
    pub const AcceptLanguage: InternId = InternId(4);
    pub const Authorization: InternId = InternId(5);
    pub const CacheControl: InternId = InternId(6);
    pub const Connection: InternId = InternId(7);
    pub const ContentLength: InternId = InternId(8);
    pub const ContentType: InternId = InternId(9);
    pub const Cookie: InternId = InternId(10);
    pub const Host: InternId = InternId(11);
    pub const IfModifiedSince: InternId = InternId(12);
    pub const IfNoneMatch: InternId = InternId(13);
    pub const Origin: InternId = InternId(14);
    pub const Referer: InternId = InternId(15);
    pub const UserAgent: InternId = InternId(16);
    pub const XForwardedFor: InternId = InternId(17);
    pub const XRequestedWith: InternId = InternId(18);

    // Response headers
    pub const AccessControlAllowOrigin: InternId = InternId(19);
    pub const AccessControlAllowMethods: InternId = InternId(20);
    pub const AccessControlAllowHeaders: InternId = InternId(21);
    pub const ContentDisposition: InternId = InternId(22);
    pub const ContentEncoding: InternId = InternId(23);
    pub const Date: InternId = InternId(24);
    pub const ETag: InternId = InternId(25);
    pub const Expires: InternId = InternId(26);
    pub const LastModified: InternId = InternId(27);
    pub const Location: InternId = InternId(28);
    pub const Server: InternId = InternId(29);
    pub const SetCookie: InternId = InternId(30);
    pub const TransferEncoding: InternId = InternId(31);
    pub const Vary: InternId = InternId(32);
    pub const WWWAuthenticate: InternId = InternId(33);
    pub const XContentTypeOptions: InternId = InternId(34);
    pub const XFrameOptions: InternId = InternId(35);
    pub const XXSSProtection: InternId = InternId(36);

    /// First ID for dynamic strings
    pub const DYNAMIC_START: u32 = 100;
}

/// Static header name lookup table
/// Maps header name bytes to InternId for O(1) lookup
static HEADER_TABLE: &[(&[u8], InternId)] = &[
    (b"accept", headers::Accept),
    (b"accept-charset", headers::AcceptCharset),
    (b"accept-encoding", headers::AcceptEncoding),
    (b"accept-language", headers::AcceptLanguage),
    (b"authorization", headers::Authorization),
    (b"cache-control", headers::CacheControl),
    (b"connection", headers::Connection),
    (b"content-length", headers::ContentLength),
    (b"content-type", headers::ContentType),
    (b"cookie", headers::Cookie),
    (b"host", headers::Host),
    (b"if-modified-since", headers::IfModifiedSince),
    (b"if-none-match", headers::IfNoneMatch),
    (b"origin", headers::Origin),
    (b"referer", headers::Referer),
    (b"user-agent", headers::UserAgent),
    (b"x-forwarded-for", headers::XForwardedFor),
    (b"x-requested-with", headers::XRequestedWith),
    (b"access-control-allow-origin", headers::AccessControlAllowOrigin),
    (b"access-control-allow-methods", headers::AccessControlAllowMethods),
    (b"access-control-allow-headers", headers::AccessControlAllowHeaders),
    (b"content-disposition", headers::ContentDisposition),
    (b"content-encoding", headers::ContentEncoding),
    (b"date", headers::Date),
    (b"etag", headers::ETag),
    (b"expires", headers::Expires),
    (b"last-modified", headers::LastModified),
    (b"location", headers::Location),
    (b"server", headers::Server),
    (b"set-cookie", headers::SetCookie),
    (b"transfer-encoding", headers::TransferEncoding),
    (b"vary", headers::Vary),
    (b"www-authenticate", headers::WWWAuthenticate),
    (b"x-content-type-options", headers::XContentTypeOptions),
    (b"x-frame-options", headers::XFrameOptions),
    (b"x-xss-protection", headers::XXSSProtection),
];

/// Lookup interned header by name (case-insensitive)
/// Returns InternId::UNKNOWN if not a known header
#[inline]
pub fn intern_header(name: &[u8]) -> InternId {
    // Fast path: check length first
    let len = name.len();
    if len < 4 || len > 32 {
        return InternId::UNKNOWN;
    }

    // Convert to lowercase for comparison
    let mut lower = [0u8; 32];
    for (i, &b) in name.iter().enumerate() {
        lower[i] = b.to_ascii_lowercase();
    }
    let lower_name = &lower[..len];

    // Linear search (could be optimized with perfect hash)
    for &(key, id) in HEADER_TABLE {
        if key == lower_name {
            return id;
        }
    }

    InternId::UNKNOWN
}

/// Get header name string from InternId
pub fn header_name(id: InternId) -> Option<&'static str> {
    match id {
        headers::Accept => Some("Accept"),
        headers::AcceptCharset => Some("Accept-Charset"),
        headers::AcceptEncoding => Some("Accept-Encoding"),
        headers::AcceptLanguage => Some("Accept-Language"),
        headers::Authorization => Some("Authorization"),
        headers::CacheControl => Some("Cache-Control"),
        headers::Connection => Some("Connection"),
        headers::ContentLength => Some("Content-Length"),
        headers::ContentType => Some("Content-Type"),
        headers::Cookie => Some("Cookie"),
        headers::Host => Some("Host"),
        headers::IfModifiedSince => Some("If-Modified-Since"),
        headers::IfNoneMatch => Some("If-None-Match"),
        headers::Origin => Some("Origin"),
        headers::Referer => Some("Referer"),
        headers::UserAgent => Some("User-Agent"),
        headers::XForwardedFor => Some("X-Forwarded-For"),
        headers::XRequestedWith => Some("X-Requested-With"),
        headers::AccessControlAllowOrigin => Some("Access-Control-Allow-Origin"),
        headers::AccessControlAllowMethods => Some("Access-Control-Allow-Methods"),
        headers::AccessControlAllowHeaders => Some("Access-Control-Allow-Headers"),
        headers::ContentDisposition => Some("Content-Disposition"),
        headers::ContentEncoding => Some("Content-Encoding"),
        headers::Date => Some("Date"),
        headers::ETag => Some("ETag"),
        headers::Expires => Some("Expires"),
        headers::LastModified => Some("Last-Modified"),
        headers::Location => Some("Location"),
        headers::Server => Some("Server"),
        headers::SetCookie => Some("Set-Cookie"),
        headers::TransferEncoding => Some("Transfer-Encoding"),
        headers::Vary => Some("Vary"),
        headers::WWWAuthenticate => Some("WWW-Authenticate"),
        headers::XContentTypeOptions => Some("X-Content-Type-Options"),
        headers::XFrameOptions => Some("X-Frame-Options"),
        headers::XXSSProtection => Some("X-XSS-Protection"),
        _ => None,
    }
}

// =============================================================================
// Dynamic String Interner
// =============================================================================

/// Thread-safe string interner for runtime deduplication
pub struct StringInterner {
    /// String -> ID mapping
    map: RwLock<HashMap<Box<str>, InternId>>,
    /// ID -> String mapping (for reverse lookup)
    strings: RwLock<Vec<Box<str>>>,
    /// Next available ID
    next_id: std::sync::atomic::AtomicU32,
}

impl StringInterner {
    /// Create new interner
    pub fn new() -> Self {
        StringInterner {
            map: RwLock::new(HashMap::new()),
            strings: RwLock::new(Vec::new()),
            next_id: std::sync::atomic::AtomicU32::new(headers::DYNAMIC_START),
        }
    }

    /// Intern a string, returning its ID
    /// If already interned, returns existing ID
    pub fn intern(&self, s: &str) -> InternId {
        // Fast path: check if already interned
        {
            let map = self.map.read().unwrap();
            if let Some(&id) = map.get(s) {
                return id;
            }
        }

        // Slow path: insert new string
        let mut map = self.map.write().unwrap();

        // Double-check after acquiring write lock
        if let Some(&id) = map.get(s) {
            return id;
        }

        let id = InternId(self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        let boxed: Box<str> = s.into();

        map.insert(boxed.clone(), id);

        let mut strings = self.strings.write().unwrap();
        let idx = (id.0 - headers::DYNAMIC_START) as usize;
        if idx >= strings.len() {
            strings.resize(idx + 1, "".into());
        }
        strings[idx] = boxed;

        id
    }

    /// Get string from ID
    pub fn get(&self, id: InternId) -> Option<String> {
        // Check static headers first
        if let Some(name) = header_name(id) {
            return Some(name.to_string());
        }

        // Check dynamic strings
        if id.0 >= headers::DYNAMIC_START {
            let strings = self.strings.read().unwrap();
            let idx = (id.0 - headers::DYNAMIC_START) as usize;
            strings.get(idx).map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Number of interned strings (excluding static headers)
    pub fn len(&self) -> usize {
        self.strings.read().unwrap().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Global interner
// =============================================================================

lazy_static::lazy_static! {
    static ref GLOBAL_INTERNER: StringInterner = StringInterner::new();
}

/// Intern string globally
pub fn intern(s: &str) -> InternId {
    GLOBAL_INTERNER.intern(s)
}

/// Get string from global interner
pub fn get_interned(id: InternId) -> Option<String> {
    GLOBAL_INTERNER.get(id)
}

// =============================================================================
// C ABI exports
// =============================================================================

/// Intern HTTP header name (case-insensitive)
#[no_mangle]
pub extern "C" fn rt_intern_header(name: *const u8, len: usize) -> u32 {
    if name.is_null() || len == 0 {
        return InternId::UNKNOWN.raw();
    }
    let bytes = unsafe { std::slice::from_raw_parts(name, len) };
    intern_header(bytes).raw()
}

/// Compare two interned IDs
#[no_mangle]
pub extern "C" fn rt_intern_eq(a: u32, b: u32) -> bool {
    a == b
}

/// Get header name from ID (returns null if unknown)
#[no_mangle]
pub extern "C" fn rt_intern_header_name(id: u32) -> *const u8 {
    header_name(InternId(id))
        .map(|s| s.as_ptr())
        .unwrap_or(std::ptr::null())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_interning() {
        assert_eq!(intern_header(b"content-type"), headers::ContentType);
        assert_eq!(intern_header(b"Content-Type"), headers::ContentType);
        assert_eq!(intern_header(b"CONTENT-TYPE"), headers::ContentType);
        assert_eq!(intern_header(b"unknown-header"), InternId::UNKNOWN);
    }

    #[test]
    fn test_header_comparison() {
        let ct1 = intern_header(b"content-type");
        let ct2 = intern_header(b"Content-Type");
        let host = intern_header(b"host");

        // Same header = same ID
        assert_eq!(ct1, ct2);
        // Different headers = different IDs
        assert_ne!(ct1, host);
    }

    #[test]
    fn test_header_name_lookup() {
        assert_eq!(header_name(headers::ContentType), Some("Content-Type"));
        assert_eq!(header_name(headers::Host), Some("Host"));
        assert_eq!(header_name(InternId::UNKNOWN), None);
    }

    #[test]
    fn test_dynamic_interning() {
        let interner = StringInterner::new();

        let id1 = interner.intern("custom-value");
        let id2 = interner.intern("custom-value");
        let id3 = interner.intern("other-value");

        // Same string = same ID
        assert_eq!(id1, id2);
        // Different string = different ID
        assert_ne!(id1, id3);

        // Reverse lookup
        assert_eq!(interner.get(id1), Some("custom-value".to_string()));
        assert_eq!(interner.get(id3), Some("other-value".to_string()));
    }

    #[test]
    fn test_intern_id_size() {
        // InternId should be small (4 bytes)
        assert_eq!(std::mem::size_of::<InternId>(), 4);
    }

    #[test]
    fn test_global_interner() {
        let id = intern("test-string");
        let s = get_interned(id);
        assert_eq!(s, Some("test-string".to_string()));
    }
}
