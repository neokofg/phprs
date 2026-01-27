//! HTTP Request (zero-copy)

use super::headers::Headers;
use super::method::Method;

/// HTTP Request — all fields are slices into the original buffer
/// Zero-copy design: no allocations during parsing
#[derive(Debug)]
pub struct Request<'a> {
    /// HTTP method (GET, POST, etc.)
    pub method: Method,

    /// Request path (e.g., "/api/users")
    pub path: &'a str,

    /// Query string without '?' (e.g., "id=1&name=test")
    pub query: Option<&'a str>,

    /// HTTP version (e.g., "HTTP/1.1")
    pub version: &'a str,

    /// Request headers
    pub headers: Headers<'a>,

    /// Request body (may be empty)
    pub body: &'a [u8],

    /// Raw URI (path + query)
    pub uri: &'a str,
}

impl<'a> Request<'a> {
    /// Create a new request
    #[inline]
    pub fn new(
        method: Method,
        uri: &'a str,
        version: &'a str,
        headers: Headers<'a>,
        body: &'a [u8],
    ) -> Self {
        let (path, query) = Self::split_uri(uri);
        Self {
            method,
            path,
            query,
            version,
            headers,
            body,
            uri,
        }
    }

    /// Split URI into path and query
    #[inline]
    fn split_uri(uri: &str) -> (&str, Option<&str>) {
        if let Some(pos) = uri.find('?') {
            let path = &uri[..pos];
            let query = if pos + 1 < uri.len() {
                Some(&uri[pos + 1..])
            } else {
                None
            };
            (path, query)
        } else {
            (uri, None)
        }
    }

    /// Get Content-Length
    #[inline]
    pub fn content_length(&self) -> Option<usize> {
        self.headers.content_length()
    }

    /// Get Content-Type
    #[inline]
    pub fn content_type(&self) -> Option<&'a str> {
        self.headers.content_type()
    }

    /// Check if request wants keep-alive
    #[inline]
    pub fn is_keep_alive(&self) -> bool {
        self.headers.is_keep_alive()
    }

    /// Get body as string (assumes UTF-8)
    #[inline]
    pub fn body_str(&self) -> Option<&'a str> {
        std::str::from_utf8(self.body).ok()
    }

    /// Check if method is GET
    #[inline]
    pub fn is_get(&self) -> bool {
        self.method == Method::Get
    }

    /// Check if method is POST
    #[inline]
    pub fn is_post(&self) -> bool {
        self.method == Method::Post
    }

    /// Get a query parameter by name
    /// Note: This is O(n) - for frequent access, parse query once
    pub fn query_param(&self, name: &str) -> Option<&'a str> {
        let query = self.query?;
        for pair in query.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = &pair[..eq_pos];
                if key == name {
                    return Some(&pair[eq_pos + 1..]);
                }
            } else if pair == name {
                return Some("");
            }
        }
        None
    }

    /// Get header value
    #[inline]
    pub fn header(&self, name: &str) -> Option<&'a str> {
        self.headers.get(name)
    }
}

/// Builder for creating test requests
#[derive(Debug, Default)]
pub struct RequestBuilder {
    method: Option<Method>,
    uri: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = Some(method);
        self
    }

    pub fn get(self, uri: &str) -> Self {
        self.method(Method::Get).uri(uri)
    }

    pub fn post(self, uri: &str) -> Self {
        self.method(Method::Post).uri(uri)
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    pub fn json(self, body: &str) -> Self {
        self.header("Content-Type", "application/json")
            .body(body.as_bytes().to_vec())
    }

    /// Build into raw HTTP request bytes
    pub fn build_bytes(&self) -> Vec<u8> {
        let method = self.method.unwrap_or(Method::Get);
        let mut buf = Vec::with_capacity(256);

        // Request line
        buf.extend_from_slice(method.as_str().as_bytes());
        buf.push(b' ');
        buf.extend_from_slice(if self.uri.is_empty() { "/" } else { &self.uri }.as_bytes());
        buf.extend_from_slice(b" HTTP/1.1\r\n");

        // Headers
        for (name, value) in &self.headers {
            buf.extend_from_slice(name.as_bytes());
            buf.extend_from_slice(b": ");
            buf.extend_from_slice(value.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        // Content-Length if body present
        if !self.body.is_empty() {
            buf.extend_from_slice(b"Content-Length: ");
            buf.extend_from_slice(self.body.len().to_string().as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        // End of headers
        buf.extend_from_slice(b"\r\n");

        // Body
        buf.extend_from_slice(&self.body);

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_uri() {
        assert_eq!(Request::split_uri("/path"), ("/path", None));
        assert_eq!(Request::split_uri("/path?query=1"), ("/path", Some("query=1")));
        assert_eq!(Request::split_uri("/path?"), ("/path", None));
        assert_eq!(Request::split_uri("/?a=b"), ("/", Some("a=b")));
    }

    #[test]
    fn test_query_param() {
        let headers = Headers::new();
        let req = Request::new(
            Method::Get,
            "/search?q=rust&page=2&active",
            "HTTP/1.1",
            headers,
            &[],
        );

        assert_eq!(req.query_param("q"), Some("rust"));
        assert_eq!(req.query_param("page"), Some("2"));
        assert_eq!(req.query_param("active"), Some(""));
        assert_eq!(req.query_param("missing"), None);
    }

    #[test]
    fn test_request_builder() {
        let bytes = RequestBuilder::new()
            .get("/api/users")
            .header("Host", "localhost")
            .build_bytes();

        let s = String::from_utf8(bytes).unwrap();
        assert!(s.starts_with("GET /api/users HTTP/1.1\r\n"));
        assert!(s.contains("Host: localhost"));
    }
}
