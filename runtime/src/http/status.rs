//! HTTP Status Codes

/// HTTP Status Code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusCode(u16);

impl StatusCode {
    // 1xx Informational
    pub const CONTINUE: Self = Self(100);
    pub const SWITCHING_PROTOCOLS: Self = Self(101);

    // 2xx Success
    pub const OK: Self = Self(200);
    pub const CREATED: Self = Self(201);
    pub const ACCEPTED: Self = Self(202);
    pub const NO_CONTENT: Self = Self(204);

    // 3xx Redirection
    pub const MOVED_PERMANENTLY: Self = Self(301);
    pub const FOUND: Self = Self(302);
    pub const SEE_OTHER: Self = Self(303);
    pub const NOT_MODIFIED: Self = Self(304);
    pub const TEMPORARY_REDIRECT: Self = Self(307);
    pub const PERMANENT_REDIRECT: Self = Self(308);

    // 4xx Client Errors
    pub const BAD_REQUEST: Self = Self(400);
    pub const UNAUTHORIZED: Self = Self(401);
    pub const FORBIDDEN: Self = Self(403);
    pub const NOT_FOUND: Self = Self(404);
    pub const METHOD_NOT_ALLOWED: Self = Self(405);
    pub const NOT_ACCEPTABLE: Self = Self(406);
    pub const CONFLICT: Self = Self(409);
    pub const GONE: Self = Self(410);
    pub const LENGTH_REQUIRED: Self = Self(411);
    pub const PAYLOAD_TOO_LARGE: Self = Self(413);
    pub const URI_TOO_LONG: Self = Self(414);
    pub const UNSUPPORTED_MEDIA_TYPE: Self = Self(415);
    pub const UNPROCESSABLE_ENTITY: Self = Self(422);
    pub const TOO_MANY_REQUESTS: Self = Self(429);

    // 5xx Server Errors
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);
    pub const NOT_IMPLEMENTED: Self = Self(501);
    pub const BAD_GATEWAY: Self = Self(502);
    pub const SERVICE_UNAVAILABLE: Self = Self(503);
    pub const GATEWAY_TIMEOUT: Self = Self(504);

    /// Create a new status code
    #[inline]
    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    /// Get the numeric code
    #[inline]
    pub const fn code(&self) -> u16 {
        self.0
    }

    /// Get the reason phrase
    #[inline]
    pub const fn reason(&self) -> &'static str {
        match self.0 {
            100 => "Continue",
            101 => "Switching Protocols",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            413 => "Payload Too Large",
            414 => "URI Too Long",
            415 => "Unsupported Media Type",
            422 => "Unprocessable Entity",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            _ => "Unknown",
        }
    }

    /// Check if status is informational (1xx)
    #[inline]
    pub const fn is_informational(&self) -> bool {
        self.0 >= 100 && self.0 < 200
    }

    /// Check if status is success (2xx)
    #[inline]
    pub const fn is_success(&self) -> bool {
        self.0 >= 200 && self.0 < 300
    }

    /// Check if status is redirection (3xx)
    #[inline]
    pub const fn is_redirection(&self) -> bool {
        self.0 >= 300 && self.0 < 400
    }

    /// Check if status is client error (4xx)
    #[inline]
    pub const fn is_client_error(&self) -> bool {
        self.0 >= 400 && self.0 < 500
    }

    /// Check if status is server error (5xx)
    #[inline]
    pub const fn is_server_error(&self) -> bool {
        self.0 >= 500 && self.0 < 600
    }

    /// Check if status is an error (4xx or 5xx)
    #[inline]
    pub const fn is_error(&self) -> bool {
        self.0 >= 400
    }
}

impl From<u16> for StatusCode {
    #[inline]
    fn from(code: u16) -> Self {
        Self(code)
    }
}

impl From<StatusCode> for u16 {
    #[inline]
    fn from(status: StatusCode) -> Self {
        status.0
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.reason())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_codes() {
        assert_eq!(StatusCode::OK.code(), 200);
        assert_eq!(StatusCode::NOT_FOUND.code(), 404);
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR.code(), 500);
    }

    #[test]
    fn test_reason() {
        assert_eq!(StatusCode::OK.reason(), "OK");
        assert_eq!(StatusCode::NOT_FOUND.reason(), "Not Found");
    }

    #[test]
    fn test_categories() {
        assert!(StatusCode::CONTINUE.is_informational());
        assert!(StatusCode::OK.is_success());
        assert!(StatusCode::FOUND.is_redirection());
        assert!(StatusCode::NOT_FOUND.is_client_error());
        assert!(StatusCode::INTERNAL_SERVER_ERROR.is_server_error());
        assert!(StatusCode::NOT_FOUND.is_error());
        assert!(StatusCode::INTERNAL_SERVER_ERROR.is_error());
        assert!(!StatusCode::OK.is_error());
    }
}
