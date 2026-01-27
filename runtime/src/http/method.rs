//! HTTP Methods

/// HTTP Request Method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Method {
    Get = 0,
    Post = 1,
    Put = 2,
    Delete = 3,
    Patch = 4,
    Head = 5,
    Options = 6,
    Connect = 7,
    Trace = 8,
}

impl Method {
    /// Parse method from bytes (zero-copy)
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        // Fast path: check length first
        match bytes.len() {
            3 => {
                if bytes.eq_ignore_ascii_case(b"GET") {
                    Some(Method::Get)
                } else if bytes.eq_ignore_ascii_case(b"PUT") {
                    Some(Method::Put)
                } else {
                    None
                }
            }
            4 => {
                if bytes.eq_ignore_ascii_case(b"POST") {
                    Some(Method::Post)
                } else if bytes.eq_ignore_ascii_case(b"HEAD") {
                    Some(Method::Head)
                } else {
                    None
                }
            }
            5 => {
                if bytes.eq_ignore_ascii_case(b"PATCH") {
                    Some(Method::Patch)
                } else if bytes.eq_ignore_ascii_case(b"TRACE") {
                    Some(Method::Trace)
                } else {
                    None
                }
            }
            6 => {
                if bytes.eq_ignore_ascii_case(b"DELETE") {
                    Some(Method::Delete)
                } else {
                    None
                }
            }
            7 => {
                if bytes.eq_ignore_ascii_case(b"OPTIONS") {
                    Some(Method::Options)
                } else if bytes.eq_ignore_ascii_case(b"CONNECT") {
                    Some(Method::Connect)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get method as string
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
            Method::Connect => "CONNECT",
            Method::Trace => "TRACE",
        }
    }

    /// Check if method typically has a body
    #[inline]
    pub const fn has_body(&self) -> bool {
        matches!(self, Method::Post | Method::Put | Method::Patch)
    }

    /// Check if method is safe (no side effects)
    #[inline]
    pub const fn is_safe(&self) -> bool {
        matches!(
            self,
            Method::Get | Method::Head | Method::Options | Method::Trace
        )
    }

    /// Check if method is idempotent
    #[inline]
    pub const fn is_idempotent(&self) -> bool {
        !matches!(self, Method::Post | Method::Patch)
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        assert_eq!(Method::from_bytes(b"GET"), Some(Method::Get));
        assert_eq!(Method::from_bytes(b"get"), Some(Method::Get));
        assert_eq!(Method::from_bytes(b"POST"), Some(Method::Post));
        assert_eq!(Method::from_bytes(b"PUT"), Some(Method::Put));
        assert_eq!(Method::from_bytes(b"DELETE"), Some(Method::Delete));
        assert_eq!(Method::from_bytes(b"PATCH"), Some(Method::Patch));
        assert_eq!(Method::from_bytes(b"HEAD"), Some(Method::Head));
        assert_eq!(Method::from_bytes(b"OPTIONS"), Some(Method::Options));
        assert_eq!(Method::from_bytes(b"INVALID"), None);
    }

    #[test]
    fn test_properties() {
        assert!(Method::Get.is_safe());
        assert!(!Method::Post.is_safe());
        assert!(Method::Get.is_idempotent());
        assert!(Method::Put.is_idempotent());
        assert!(!Method::Post.is_idempotent());
        assert!(Method::Post.has_body());
        assert!(!Method::Get.has_body());
    }
}
