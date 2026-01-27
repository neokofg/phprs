//! HTTP Module
//!
//! Low-level HTTP primitives for building web frameworks.
//!
//! # Features
//!
//! - **Zero-copy parsing**: Request fields are slices into the original buffer
//! - **Interned headers**: O(1) header name comparison using InternedString
//! - **Inline storage**: Up to 16 headers stored without heap allocation
//! - **Keep-alive support**: Connection reuse for HTTP/1.1
//!
//! # Design Philosophy
//!
//! This module provides building blocks, not a complete framework:
//! - No routing (use a router crate or build your own)
//! - No middleware system (compose handlers as you like)
//! - No session/cookie abstractions (implement per your needs)
//!
//! # Example
//!
//! ```ignore
//! use phprs_runtime::http::{serve, Request, Response};
//!
//! serve("127.0.0.1:8080", |req| {
//!     match (req.method, req.path) {
//!         (Method::Get, "/") => Response::ok().text("Hello!"),
//!         (Method::Get, "/api/users") => Response::ok().json(r#"{"users":[]}"#),
//!         _ => Response::not_found().text("Not Found"),
//!     }
//! }).unwrap();
//! ```

pub mod method;
pub mod status;
pub mod headers;
pub mod request;
pub mod response;
pub mod parser;
pub mod server;

// Re-exports
pub use method::Method;
pub use status::StatusCode;
pub use headers::Headers;
pub use request::{Request, RequestBuilder};
pub use response::Response;
pub use parser::{HttpParser, ParseError, ParseResult, ParserConfig, parse_request};
pub use server::{HttpServer, ServerConfig, Connection, ConnectionError, serve, serve_threaded};

use crate::string::SmartString;

// =============================================================================
// C ABI Exports for PHP Runtime
// =============================================================================

/// Create HTTP server (returns opaque pointer)
#[no_mangle]
pub extern "C" fn rt_http_server_create(host: &SmartString, port: u16) -> *mut HttpServer {
    let addr = format!("{}:{}", host.as_str(), port);
    match HttpServer::bind(&addr) {
        Ok(server) => Box::into_raw(Box::new(server)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Destroy HTTP server
#[no_mangle]
pub extern "C" fn rt_http_server_destroy(server: *mut HttpServer) {
    if !server.is_null() {
        unsafe { drop(Box::from_raw(server)); }
    }
}

/// Accept connection (blocking)
#[no_mangle]
pub extern "C" fn rt_http_accept(server: *mut HttpServer) -> *mut Connection {
    if server.is_null() {
        return std::ptr::null_mut();
    }

    let server = unsafe { &*server };
    match server.accept() {
        Ok(conn) => Box::into_raw(Box::new(conn)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Destroy connection
#[no_mangle]
pub extern "C" fn rt_http_connection_destroy(conn: *mut Connection) {
    if !conn.is_null() {
        unsafe { drop(Box::from_raw(conn)); }
    }
}

/// Create response
#[no_mangle]
pub extern "C" fn rt_http_response_new(status: u16) -> *mut Response {
    Box::into_raw(Box::new(Response::new(StatusCode::new(status))))
}

/// Set response header
#[no_mangle]
pub extern "C" fn rt_http_response_header(
    resp: *mut Response,
    name: &SmartString,
    value: &SmartString,
) {
    if resp.is_null() {
        return;
    }
    let resp = unsafe { &mut *resp };
    // We need to rebuild response with new header
    // This is a bit awkward due to builder pattern
    let name_clone = name.clone();
    let value_clone = value.clone();
    resp.headers.push((name_clone, value_clone));
}


/// Set response body
#[no_mangle]
pub extern "C" fn rt_http_response_body(resp: *mut Response, body: &SmartString) {
    if resp.is_null() {
        return;
    }
    let resp = unsafe { &mut *resp };
    resp.body = body.as_bytes().to_vec();
}

/// Send response and destroy it
#[no_mangle]
pub extern "C" fn rt_http_respond(conn: *mut Connection, resp: *mut Response) -> bool {
    if conn.is_null() || resp.is_null() {
        return false;
    }

    let conn = unsafe { &mut *conn };
    let resp = unsafe { Box::from_raw(resp) };

    conn.write_response(&resp).is_ok()
}

/// Get request method as int
#[no_mangle]
pub extern "C" fn rt_http_request_method(req: &Request) -> u8 {
    req.method as u8
}

/// Get request path
#[no_mangle]
pub extern "C" fn rt_http_request_path(req: &Request) -> SmartString {
    SmartString::from_str(req.path)
}

/// Get request query string
#[no_mangle]
pub extern "C" fn rt_http_request_query(req: &Request) -> SmartString {
    SmartString::from_str(req.query.unwrap_or(""))
}

/// Get request header
#[no_mangle]
pub extern "C" fn rt_http_request_header(req: &Request, name: &SmartString) -> SmartString {
    SmartString::from_str(req.header(name.as_str()).unwrap_or(""))
}

/// Get request body as string
#[no_mangle]
pub extern "C" fn rt_http_request_body(req: &Request) -> SmartString {
    SmartString::from_str(req.body_str().unwrap_or(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_roundtrip() {
        // Build request
        let raw = RequestBuilder::new()
            .post("/api/users")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(r#"{"name":"Alice"}"#)
            .build_bytes();

        // Parse request
        let (req, _) = parse_request(&raw).unwrap();

        assert_eq!(req.method, Method::Post);
        assert_eq!(req.path, "/api/users");
        assert_eq!(req.content_type(), Some("application/json"));
        assert_eq!(req.body_str(), Some(r#"{"name":"Alice"}"#));

        // Build response
        let resp = Response::created()
            .header("Location", "/api/users/1")
            .json(r#"{"id":1,"name":"Alice"}"#);

        let bytes = resp.to_bytes();
        let s = String::from_utf8(bytes).unwrap();

        assert!(s.starts_with("HTTP/1.1 201 Created"));
        assert!(s.contains("Location: /api/users/1"));
        assert!(s.contains(r#"{"id":1,"name":"Alice"}"#));
    }

    #[test]
    fn test_method_enum() {
        assert_eq!(Method::Get as u8, 0);
        assert_eq!(Method::Post as u8, 1);
        assert_eq!(Method::Put as u8, 2);
        assert_eq!(Method::Delete as u8, 3);
    }

    #[test]
    fn test_status_code() {
        assert_eq!(StatusCode::OK.code(), 200);
        assert_eq!(StatusCode::NOT_FOUND.code(), 404);
        assert!(StatusCode::OK.is_success());
        assert!(StatusCode::NOT_FOUND.is_client_error());
    }
}
