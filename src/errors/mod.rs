use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum CompileError {
    #[error("Lexer error: {message}")]
    #[diagnostic(code(phprs::lexer))]
    LexerError {
        message: String,
        #[label("here")]
        span: (usize, usize),
    },

    #[error("Parser error: {message}")]
    #[diagnostic(code(phprs::parser))]
    ParserError {
        message: String,
        #[label("here")]
        span: (usize, usize),
    },

    #[error("Type error: {message}")]
    #[diagnostic(code(phprs::type_check))]
    TypeError {
        message: String,
        #[label("here")]
        span: (usize, usize),
    },

    #[error("Ownership error: {message}")]
    #[diagnostic(code(phprs::ownership))]
    OwnershipError {
        message: String,
        #[label("value was moved here")]
        move_span: (usize, usize),
        #[label("used here after move")]
        use_span: Option<(usize, usize)>,
    },

    #[error("Codegen error: {message}")]
    #[diagnostic(code(phprs::codegen))]
    CodegenError { message: String },
}

pub type Result<T> = std::result::Result<T, CompileError>;
