//! HTTP Response Builder

use super::status::StatusCode;
use crate::string::SmartString;

/// HTTP Response
#[derive(Debug, Clone)]
pub struct Response {
    /// Status code
    pub status: StatusCode,

    /// Response headers as (name, value) pairs
    pub headers: Vec<(SmartString, SmartString)>,

    /// Response body
    pub body: Vec<u8>,
}

impl Response {
    /// Create a new response with status code
    #[inline]
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: Vec::with_capacity(8),
            body: Vec::new(),
        }
    }

    /// Create 200 OK response
    #[inline]
    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    /// Create 201 Created response
    #[inline]
    pub fn created() -> Self {
        Self::new(StatusCode::CREATED)
    }

    /// Create 204 No Content response
    #[inline]
    pub fn no_content() -> Self {
        Self::new(StatusCode::NO_CONTENT)
    }

    /// Create 400 Bad Request response
    #[inline]
    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST)
    }

    /// Create 401 Unauthorized response
    #[inline]
    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED)
    }

    /// Create 403 Forbidden response
    #[inline]
    pub fn forbidden() -> Self {
        Self::new(StatusCode::FORBIDDEN)
    }

    /// Create 404 Not Found response
    #[inline]
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    /// Create 500 Internal Server Error response
    #[inline]
    pub fn internal_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Set status code
    #[inline]
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Add a header
    #[inline]
    pub fn header(mut self, name: impl Into<SmartString>, value: impl Into<SmartString>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Set Content-Type header
    #[inline]
    pub fn content_type(self, ct: &str) -> Self {
        self.header("Content-Type", ct)
    }

    /// Set body from bytes
    #[inline]
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Set body from string
    #[inline]
    pub fn body_str(self, body: &str) -> Self {
        self.body(body.as_bytes().to_vec())
    }

    /// Set JSON body
    #[inline]
    pub fn json(self, body: &str) -> Self {
        self.content_type("application/json")
            .body(body.as_bytes().to_vec())
    }

    /// Set HTML body
    #[inline]
    pub fn html(self, body: &str) -> Self {
        self.content_type("text/html; charset=utf-8")
            .body(body.as_bytes().to_vec())
    }

    /// Set plain text body
    #[inline]
    pub fn text(self, body: &str) -> Self {
        self.content_type("text/plain; charset=utf-8")
            .body(body.as_bytes().to_vec())
    }

    /// Get body length
    #[inline]
    pub fn body_len(&self) -> usize {
        self.body.len()
    }

    /// Get body as slice
    #[inline]
    pub fn body_bytes(&self) -> &[u8] {
        &self.body
    }

    /// Serialize response to bytes (HTTP/1.1 format)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(256 + self.body.len());

        // Status line
        buf.extend_from_slice(b"HTTP/1.1 ");
        write_u16(&mut buf, self.status.code());
        buf.push(b' ');
        buf.extend_from_slice(self.status.reason().as_bytes());
        buf.extend_from_slice(b"\r\n");

        // Headers
        for (name, value) in &self.headers {
            buf.extend_from_slice(name.as_bytes());
            buf.extend_from_slice(b": ");
            buf.extend_from_slice(value.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        // Content-Length (always add for non-empty body)
        if !self.body.is_empty() {
            buf.extend_from_slice(b"Content-Length: ");
            write_usize(&mut buf, self.body.len());
            buf.extend_from_slice(b"\r\n");
        }

        // End of headers
        buf.extend_from_slice(b"\r\n");

        // Body
        buf.extend_from_slice(&self.body);

        buf
    }

    /// Write response to a buffer (avoids allocation if buffer is large enough)
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        buf.clear();
        buf.reserve(256 + self.body.len());

        // Status line
        buf.extend_from_slice(b"HTTP/1.1 ");
        write_u16(buf, self.status.code());
        buf.push(b' ');
        buf.extend_from_slice(self.status.reason().as_bytes());
        buf.extend_from_slice(b"\r\n");

        // Headers
        for (name, value) in &self.headers {
            buf.extend_from_slice(name.as_bytes());
            buf.extend_from_slice(b": ");
            buf.extend_from_slice(value.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        // Content-Length
        if !self.body.is_empty() {
            buf.extend_from_slice(b"Content-Length: ");
            write_usize(buf, self.body.len());
            buf.extend_from_slice(b"\r\n");
        }

        // End of headers
        buf.extend_from_slice(b"\r\n");

        // Body
        buf.extend_from_slice(&self.body);
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::ok()
    }
}

/// Fast u16 to bytes
#[inline]
fn write_u16(buf: &mut Vec<u8>, n: u16) {
    if n < 10 {
        buf.push(b'0' + n as u8);
    } else if n < 100 {
        buf.push(b'0' + (n / 10) as u8);
        buf.push(b'0' + (n % 10) as u8);
    } else if n < 1000 {
        buf.push(b'0' + (n / 100) as u8);
        buf.push(b'0' + ((n / 10) % 10) as u8);
        buf.push(b'0' + (n % 10) as u8);
    } else {
        // For HTTP status codes, max is 599
        let mut tmp = itoa::Buffer::new();
        buf.extend_from_slice(tmp.format(n).as_bytes());
    }
}

/// Fast usize to bytes
#[inline]
fn write_usize(buf: &mut Vec<u8>, n: usize) {
    let mut tmp = itoa::Buffer::new();
    buf.extend_from_slice(tmp.format(n).as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_basic() {
        let resp = Response::ok()
            .header("X-Custom", "value")
            .body_str("Hello");

        let bytes = resp.to_bytes();
        let s = String::from_utf8(bytes).unwrap();

        assert!(s.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(s.contains("X-Custom: value\r\n"));
        assert!(s.contains("Content-Length: 5\r\n"));
        assert!(s.ends_with("\r\n\r\nHello"));
    }

    #[test]
    fn test_response_json() {
        let resp = Response::ok().json(r#"{"ok":true}"#);

        let bytes = resp.to_bytes();
        let s = String::from_utf8(bytes).unwrap();

        assert!(s.contains("Content-Type: application/json\r\n"));
        assert!(s.ends_with(r#"{"ok":true}"#));
    }

    #[test]
    fn test_response_not_found() {
        let resp = Response::not_found().text("Page not found");

        assert_eq!(resp.status, StatusCode::NOT_FOUND);
        assert_eq!(resp.body_len(), 14);
    }

    #[test]
    fn test_write_u16() {
        let mut buf = Vec::new();
        write_u16(&mut buf, 200);
        assert_eq!(&buf, b"200");

        buf.clear();
        write_u16(&mut buf, 5);
        assert_eq!(&buf, b"5");

        buf.clear();
        write_u16(&mut buf, 99);
        assert_eq!(&buf, b"99");
    }
}
