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
pub mod string;

pub use arena::{Arena, ArenaStats, thread_alloc, thread_alloc_val, thread_arena_reset, thread_arena_stats};
pub use array::{PhpArray, ArrayKey, PhpValue};
pub use http::{
    Method, StatusCode, Headers, Request, Response,
    HttpServer, ServerConfig, Connection,
    HttpParser, ParseError, parse_request,
    serve, serve_threaded,
};
pub use intern::{InternId, StringInterner, headers, intern, intern_header, get_interned, header_name};
pub use json::{decode as json_decode, encode as json_encode, JsonError};
pub use fs::{FsError, FileHandle, OpenMode, SeekOrigin, FsResult};
pub use string::SmartString;
