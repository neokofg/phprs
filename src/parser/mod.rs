//! Parser module - converts tokens into AST

#![allow(clippy::missing_errors_doc)]

mod class;
mod expr;
mod stmt;

use crate::ast::{Function, Param, Program, Span, Type};
use crate::errors::CompileError;
use crate::lexer::{SpannedToken, TokenKind};
use miette::Result;

/// Parse tokens into an AST.
pub fn parse(tokens: Vec<SpannedToken>) -> Result<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    const fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> &SpannedToken {
        self.tokens
            .get(self.pos)
            .unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn peek(&self) -> TokenKind {
        self.current().kind
    }

    #[allow(dead_code)]
    fn peek_ahead(&self, n: usize) -> TokenKind {
        self.tokens
            .get(self.pos + n)
            .map_or(TokenKind::Eof, |t| t.kind)
    }

    fn advance(&mut self) -> &SpannedToken {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&SpannedToken> {
        if self.peek() == kind {
            Ok(self.advance())
        } else {
            Err(CompileError::ParserError {
                message: format!("Expected {:?}, found {:?}", kind, self.peek()),
                span: self.current().span,
            }
            .into())
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek() == kind
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn span(&self) -> Span {
        self.current().span.into()
    }

    // === Program parsing ===

    fn parse_program(&mut self) -> Result<Program> {
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut top_level_stmts = Vec::new();

        while !self.check(TokenKind::Eof) {
            match self.peek() {
                TokenKind::Fn => {
                    functions.push(self.parse_function()?);
                }
                TokenKind::Class | TokenKind::Abstract | TokenKind::Final => {
                    classes.push(self.parse_class()?);
                }
                TokenKind::Interface => {
                    return Err(CompileError::ParserError {
                        message: "Interfaces are not yet fully supported".to_string(),
                        span: self.current().span,
                    }
                    .into());
                }
                TokenKind::Variable
                | TokenKind::Echo
                | TokenKind::If
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Return
                | TokenKind::LBrace
                | TokenKind::Identifier => {
                    top_level_stmts.push(self.parse_stmt()?);
                }
                _ => {
                    return Err(CompileError::ParserError {
                        message: format!("Unexpected token at top level: {:?}", self.peek()),
                        span: self.current().span,
                    }
                    .into());
                }
            }
        }

        if !top_level_stmts.is_empty() {
            let has_main = functions.iter().any(|f| f.name == "main");
            if has_main {
                return Err(CompileError::ParserError {
                    message: "Cannot have both a main function and top-level code".to_string(),
                    span: self.current().span,
                }
                .into());
            }

            let main_fn = Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: top_level_stmts,
                span: Span::default(),
            };
            functions.push(main_fn);
        }

        Ok(Program { functions, classes })
    }

    // === Function parsing ===

    fn parse_function(&mut self) -> Result<Function> {
        let start = self.span();
        self.expect(TokenKind::Fn)?;

        let name_token = self.expect(TokenKind::Identifier)?;
        let name = name_token.text.clone();

        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;

        let return_type = if self.match_token(TokenKind::Colon) {
            self.parse_type()?
        } else {
            Type::Unknown
        };

        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block_contents()?;
        let end = self.span();
        self.expect(TokenKind::RBrace)?;

        Ok(Function {
            name,
            params,
            return_type,
            body,
            span: start.merge(end),
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();

        if self.check(TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            params.push(self.parse_param()?);
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param> {
        let start = self.span();
        let is_ref = self.match_token(TokenKind::Ampersand);

        let name_token = self.expect(TokenKind::Variable)?;
        let name = name_token.text[1..].to_string();

        let ty = if self.match_token(TokenKind::Colon) {
            self.parse_type()?
        } else {
            Type::Unknown
        };

        Ok(Param {
            name,
            ty,
            is_ref,
            span: start.merge(self.span()),
        })
    }

    fn parse_type(&mut self) -> Result<Type> {
        let is_ref = self.match_token(TokenKind::Ampersand);
        let is_nullable = self.match_token(TokenKind::Not);

        let base_type = match self.peek() {
            TokenKind::TypeInt => {
                self.advance();
                Type::Int
            }
            TokenKind::TypeFloat => {
                self.advance();
                Type::Float
            }
            TokenKind::TypeBool => {
                self.advance();
                Type::Bool
            }
            TokenKind::TypeString => {
                self.advance();
                Type::String
            }
            TokenKind::TypeVoid => {
                self.advance();
                Type::Void
            }
            TokenKind::SelfKw => {
                self.advance();
                Type::SelfType
            }
            TokenKind::Static => {
                self.advance();
                Type::StaticType
            }
            TokenKind::Identifier => {
                let name_token = self.advance().clone();
                let name = name_token.text;

                if name == "array" && self.check(TokenKind::Lt) {
                    self.advance();
                    let inner = self.parse_type()?;
                    self.expect(TokenKind::Gt)?;
                    Type::Array(Box::new(inner))
                } else {
                    Type::Class(name)
                }
            }
            _ => {
                return Err(CompileError::ParserError {
                    message: format!("Expected type, found {:?}", self.peek()),
                    span: self.current().span,
                }
                .into());
            }
        };

        let typed = if is_nullable {
            Type::Nullable(Box::new(base_type))
        } else {
            base_type
        };

        if is_ref {
            Ok(Type::Ref(Box::new(typed)))
        } else {
            Ok(typed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_function() {
        let source = r#"<?php
fn main() {
    echo "Hello";
}
"#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
    }

    #[test]
    fn test_parse_binary_expr() {
        let source = "<?php fn main() { $x: int = 1 + 2 * 3; }";
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
    }
}
