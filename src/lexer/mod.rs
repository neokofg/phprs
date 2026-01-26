#![allow(clippy::missing_errors_doc)]

mod token;

pub use token::TokenKind;

use crate::errors::CompileError;
use logos::Logos;
use miette::Result;

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub kind: TokenKind,
    pub span: (usize, usize),
    pub text: String,
}

pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>> {
    let mut lexer = TokenKind::lexer(source);
    let mut tokens = Vec::new();
    let mut php_mode = false;

    while let Some(result) = lexer.next() {
        let span = lexer.span();
        let text = lexer.slice().to_string();

        match result {
            Ok(kind) => {
                // Handle PHP open tag
                if kind == TokenKind::PhpOpen {
                    php_mode = true;
                    continue;
                }

                // Skip comments
                if matches!(kind, TokenKind::Comment) {
                    continue;
                }

                if php_mode {
                    tokens.push(SpannedToken {
                        kind,
                        span: (span.start, span.end),
                        text,
                    });
                }
            }
            Err(()) => {
                if php_mode {
                    return Err(CompileError::LexerError {
                        message: format!("Unexpected character: {}", lexer.slice()),
                        span: (span.start, span.end),
                    }
                    .into());
                }
            }
        }
    }

    tokens.push(SpannedToken {
        kind: TokenKind::Eof,
        span: (source.len(), source.len()),
        text: String::new(),
    });

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokenize() {
        let source = r#"<?php
fn main() {
    echo "Hello";
}
"#;
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Fn));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Identifier));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Echo));
    }

    #[test]
    fn test_variable() {
        let source = "<?php $x = 42;";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Variable));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Integer));
    }
}
