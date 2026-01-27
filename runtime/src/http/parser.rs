//! Zero-copy HTTP/1.1 Parser

use super::headers::Headers;
use super::method::Method;
use super::request::Request;

/// HTTP Parse Error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Incomplete request (need more data)
    Incomplete,
    /// Invalid method
    InvalidMethod,
    /// Invalid request line
    InvalidRequestLine,
    /// Invalid header
    InvalidHeader,
    /// Header name too long
    HeaderNameTooLong,
    /// Header value too long
    HeaderValueTooLong,
    /// Too many headers
    TooManyHeaders,
    /// Invalid HTTP version
    InvalidVersion,
    /// URI too long
    UriTooLong,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Incomplete => write!(f, "Incomplete request"),
            ParseError::InvalidMethod => write!(f, "Invalid HTTP method"),
            ParseError::InvalidRequestLine => write!(f, "Invalid request line"),
            ParseError::InvalidHeader => write!(f, "Invalid header"),
            ParseError::HeaderNameTooLong => write!(f, "Header name too long"),
            ParseError::HeaderValueTooLong => write!(f, "Header value too long"),
            ParseError::TooManyHeaders => write!(f, "Too many headers"),
            ParseError::InvalidVersion => write!(f, "Invalid HTTP version"),
            ParseError::UriTooLong => write!(f, "URI too long"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Result of parsing
pub type ParseResult<T> = Result<T, ParseError>;

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum URI length (default: 8192)
    pub max_uri_len: usize,
    /// Maximum header name length (default: 256)
    pub max_header_name_len: usize,
    /// Maximum header value length (default: 8192)
    pub max_header_value_len: usize,
    /// Maximum number of headers (default: 64)
    pub max_headers: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_uri_len: 8192,
            max_header_name_len: 256,
            max_header_value_len: 8192,
            max_headers: 64,
        }
    }
}

/// Zero-copy HTTP parser
///
/// All returned strings are slices into the input buffer.
/// No allocations during parsing (except Headers overflow).
pub struct HttpParser {
    config: ParserConfig,
}

impl HttpParser {
    /// Create parser with default config
    #[inline]
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }

    /// Create parser with custom config
    #[inline]
    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    /// Parse HTTP request from buffer
    ///
    /// Returns (Request, bytes_consumed) on success.
    /// Returns Err(Incomplete) if more data needed.
    pub fn parse<'a>(&self, buf: &'a [u8]) -> ParseResult<(Request<'a>, usize)> {
        // Find end of request line
        let request_line_end = find_crlf(buf).ok_or(ParseError::Incomplete)?;
        let request_line = &buf[..request_line_end];

        // Parse request line
        let (method, uri, version) = self.parse_request_line(request_line)?;

        // Parse headers
        let mut pos = request_line_end + 2; // skip \r\n
        let mut headers = Headers::new();
        let mut header_count = 0;

        loop {
            // Check for end of headers
            if pos + 1 < buf.len() && buf[pos] == b'\r' && buf[pos + 1] == b'\n' {
                pos += 2;
                break;
            }

            // Find end of header line
            let header_end = find_crlf(&buf[pos..]).ok_or(ParseError::Incomplete)?;
            let header_line = &buf[pos..pos + header_end];

            // Parse header
            if !header_line.is_empty() {
                let (name, value) = self.parse_header(header_line)?;
                headers.insert(name, value);
                header_count += 1;

                if header_count > self.config.max_headers {
                    return Err(ParseError::TooManyHeaders);
                }
            }

            pos += header_end + 2; // skip \r\n
        }

        // Get body
        let content_length = headers.content_length().unwrap_or(0);
        let body_end = pos + content_length;

        if buf.len() < body_end {
            return Err(ParseError::Incomplete);
        }

        let body = &buf[pos..body_end];

        let request = Request::new(method, uri, version, headers, body);
        Ok((request, body_end))
    }

    /// Parse request line: "GET /path HTTP/1.1"
    fn parse_request_line<'a>(&self, line: &'a [u8]) -> ParseResult<(Method, &'a str, &'a str)> {
        // Find first space (after method)
        let method_end = memchr(b' ', line).ok_or(ParseError::InvalidRequestLine)?;
        let method_bytes = &line[..method_end];
        let method = Method::from_bytes(method_bytes).ok_or(ParseError::InvalidMethod)?;

        // Find second space (after URI)
        let uri_start = method_end + 1;
        let uri_end = memchr(b' ', &line[uri_start..])
            .map(|p| p + uri_start)
            .ok_or(ParseError::InvalidRequestLine)?;

        let uri_bytes = &line[uri_start..uri_end];
        if uri_bytes.len() > self.config.max_uri_len {
            return Err(ParseError::UriTooLong);
        }
        let uri = std::str::from_utf8(uri_bytes).map_err(|_| ParseError::InvalidRequestLine)?;

        // Version
        let version_bytes = &line[uri_end + 1..];
        if !version_bytes.starts_with(b"HTTP/") {
            return Err(ParseError::InvalidVersion);
        }
        let version = std::str::from_utf8(version_bytes).map_err(|_| ParseError::InvalidVersion)?;

        Ok((method, uri, version))
    }

    /// Parse header line: "Content-Type: application/json"
    fn parse_header<'a>(&self, line: &'a [u8]) -> ParseResult<(&'a str, &'a str)> {
        let colon = memchr(b':', line).ok_or(ParseError::InvalidHeader)?;

        let name_bytes = &line[..colon];
        if name_bytes.len() > self.config.max_header_name_len {
            return Err(ParseError::HeaderNameTooLong);
        }
        let name = std::str::from_utf8(name_bytes).map_err(|_| ParseError::InvalidHeader)?;

        // Skip colon and optional whitespace
        let mut value_start = colon + 1;
        while value_start < line.len() && (line[value_start] == b' ' || line[value_start] == b'\t') {
            value_start += 1;
        }

        let value_bytes = &line[value_start..];
        if value_bytes.len() > self.config.max_header_value_len {
            return Err(ParseError::HeaderValueTooLong);
        }
        let value = std::str::from_utf8(value_bytes).map_err(|_| ParseError::InvalidHeader)?;

        Ok((name, value))
    }
}

impl Default for HttpParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Find \r\n in buffer
#[inline]
fn find_crlf(buf: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 1 < buf.len() {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Find byte in buffer (simple memchr)
#[inline]
fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

/// Convenience function: parse request from buffer
#[inline]
pub fn parse_request(buf: &[u8]) -> ParseResult<(Request<'_>, usize)> {
    HttpParser::new().parse(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_get() {
        let raw = b"GET /path HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let (req, consumed) = parse_request(raw).unwrap();

        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/path");
        assert_eq!(req.version, "HTTP/1.1");
        assert_eq!(req.header("Host"), Some("localhost"));
        assert_eq!(consumed, raw.len());
    }

    #[test]
    fn test_parse_with_query() {
        let raw = b"GET /search?q=rust&page=1 HTTP/1.1\r\n\r\n";
        let (req, _) = parse_request(raw).unwrap();

        assert_eq!(req.path, "/search");
        assert_eq!(req.query, Some("q=rust&page=1"));
        assert_eq!(req.query_param("q"), Some("rust"));
        assert_eq!(req.query_param("page"), Some("1"));
    }

    #[test]
    fn test_parse_post_with_body() {
        // {"id":1} is 8 bytes, so Content-Length: 8
        let raw = b"POST /api/users HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 8\r\n\r\n{\"id\":1}extra";
        let (req, consumed) = parse_request(raw).unwrap();

        assert_eq!(req.method, Method::Post);
        assert_eq!(req.path, "/api/users");
        assert_eq!(req.content_type(), Some("application/json"));
        assert_eq!(req.body, b"{\"id\":1}"); // only 8 bytes per Content-Length
        assert_eq!(consumed, raw.len() - 5); // "extra" not consumed
    }

    #[test]
    fn test_parse_incomplete() {
        let raw = b"GET /path HTTP/1.1\r\nHost: local";
        let result = parse_request(raw);
        assert!(matches!(result, Err(ParseError::Incomplete)));
    }

    #[test]
    fn test_parse_multiple_headers() {
        let raw = b"GET / HTTP/1.1\r\n\
            Host: localhost\r\n\
            Accept: application/json\r\n\
            User-Agent: test\r\n\
            X-Custom: value\r\n\
            \r\n";

        let (req, _) = parse_request(raw).unwrap();

        assert_eq!(req.header("Host"), Some("localhost"));
        assert_eq!(req.header("Accept"), Some("application/json"));
        assert_eq!(req.header("User-Agent"), Some("test"));
        assert_eq!(req.header("X-Custom"), Some("value"));
    }

    #[test]
    fn test_parse_invalid_method() {
        let raw = b"INVALID /path HTTP/1.1\r\n\r\n";
        let result = parse_request(raw);
        assert!(matches!(result, Err(ParseError::InvalidMethod)));
    }

    #[test]
    fn test_find_crlf() {
        assert_eq!(find_crlf(b"hello\r\nworld"), Some(5));
        assert_eq!(find_crlf(b"hello"), None);
        assert_eq!(find_crlf(b"\r\n"), Some(0));
    }

    #[test]
    fn test_keep_alive() {
        let raw = b"GET / HTTP/1.1\r\nConnection: keep-alive\r\n\r\n";
        let (req, _) = parse_request(raw).unwrap();
        assert!(req.is_keep_alive());

        let raw = b"GET / HTTP/1.1\r\nConnection: close\r\n\r\n";
        let (req, _) = parse_request(raw).unwrap();
        assert!(!req.is_keep_alive());
    }
}
