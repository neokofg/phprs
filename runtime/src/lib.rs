//! PHPRS Runtime Library
//!
//! High-performance runtime for the PHPRS compiler.
//! Provides optimized implementations for strings, arrays, JSON, HTTP, and filesystem.

#![allow(clippy::missing_safety_doc)]

pub mod arena;
pub mod array;
pub mod fs;
pub mod http;
pub mod intern;
pub mod json;
pub mod math;
pub mod string;
pub mod types;

pub use arena::{
    thread_alloc, thread_alloc_val, thread_arena_reset, thread_arena_stats, Arena, ArenaStats,
};
pub use array::{ArrayKey, PhpArray, PhpValue};
pub use fs::{FileHandle, FsError, FsResult, OpenMode, SeekOrigin};
pub use http::{
    parse_request, serve, serve_threaded, Connection, Headers, HttpParser, HttpServer, Method,
    ParseError, Request, Response, ServerConfig, StatusCode,
};
pub use intern::{
    get_interned, header_name, headers, intern, intern_header, InternId, StringInterner,
};
pub use json::{decode as json_decode, encode as json_encode, JsonError};
pub use string::SmartString;
