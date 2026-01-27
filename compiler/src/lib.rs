pub mod ast;
pub mod codegen;
pub mod errors;
pub mod lexer;
pub mod ownership;
pub mod parser;
pub mod resolver;
pub mod stdlib;
pub mod types;

pub use errors::CompileError;

// Re-export runtime for linking
pub use phprs_runtime;
