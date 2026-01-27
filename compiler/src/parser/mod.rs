//! Parser module - converts tokens into AST

#![allow(clippy::missing_errors_doc)]

mod class;
mod expr;
mod namespace;
mod stmt;

use crate::ast::{
    Attribute, AttributeArg, Attributes, CompilationUnit, Function, Param, Program, Span, Type,
};
use crate::errors::CompileError;
use crate::lexer::{SpannedToken, TokenKind};
use miette::Result;

/// Parse tokens into an AST.
#[allow(dead_code)]
pub fn parse(tokens: Vec<SpannedToken>) -> Result<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

/// Parse tokens into a compilation unit (single file).
pub fn parse_unit(tokens: Vec<SpannedToken>) -> Result<CompilationUnit> {
    let mut parser = Parser::new(tokens);
    parser.parse_compilation_unit()
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

    /// Parse a compilation unit (single file with namespace and uses)
    fn parse_compilation_unit(&mut self) -> Result<CompilationUnit> {
        // 1. Parse optional namespace declaration (must be first after <?php)
        let namespace = if self.check(TokenKind::Namespace) {
            Some(self.parse_namespace()?)
        } else {
            None
        };

        // 2. Parse top-level use declarations
        let mut uses = Vec::new();
        while self.check(TokenKind::Use) {
            uses.push(self.parse_use_declaration()?);
        }

        // 3. Parse functions, classes, and traits
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut traits = Vec::new();
        let mut top_level_stmts = Vec::new();

        while !self.check(TokenKind::Eof) {
            // Parse attributes if present
            let attrs = self.parse_attributes()?;

            match self.peek() {
                TokenKind::Fn => {
                    functions.push(self.parse_function_with_attrs(attrs)?);
                }
                TokenKind::Class | TokenKind::Abstract | TokenKind::Final => {
                    classes.push(self.parse_class_with_attrs(attrs)?);
                }
                TokenKind::Trait => {
                    traits.push(self.parse_trait_with_attrs(attrs)?);
                }
                TokenKind::Interface => {
                    return Err(CompileError::ParserError {
                        message: "Interfaces are not yet fully supported".to_string(),
                        span: self.current().span,
                    }
                    .into());
                }
                TokenKind::Namespace => {
                    return Err(CompileError::ParserError {
                        message: "Namespace declaration must be at the beginning of the file"
                            .to_string(),
                        span: self.current().span,
                    }
                    .into());
                }
                TokenKind::Use => {
                    return Err(CompileError::ParserError {
                        message: "Use declarations must appear before any other code".to_string(),
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
                    if !attrs.is_empty() {
                        return Err(CompileError::ParserError {
                            message: "Attributes cannot be applied to statements".to_string(),
                            span: self.current().span,
                        }
                        .into());
                    }
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

        // Wrap top-level statements in main function
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
                attributes: Attributes::default(),
                span: Span::default(),
            };
            functions.push(main_fn);
        }

        Ok(CompilationUnit {
            namespace,
            uses,
            functions,
            classes,
            traits,
            file_path: None,
        })
    }

    /// Parse a program (backwards compatible - wraps compilation unit)
    #[allow(dead_code)]
    fn parse_program(&mut self) -> Result<Program> {
        let unit = self.parse_compilation_unit()?;
        Ok(Program::from_unit(unit))
    }

    // === Attribute parsing ===

    /// Parse a sequence of attributes: #[Attr1] #[Attr2(arg)]
    fn parse_attributes(&mut self) -> Result<Attributes> {
        let mut attrs = Attributes::new();

        while self.check(TokenKind::HashBracket) {
            attrs.push(self.parse_attribute()?);
        }

        Ok(attrs)
    }

    /// Parse single attribute: #[Name(arg1, key: value)]
    fn parse_attribute(&mut self) -> Result<Attribute> {
        let start = self.span();
        self.expect(TokenKind::HashBracket)?;

        let name_token = self.expect(TokenKind::Identifier)?;
        let name = name_token.text.clone();

        let args = if self.match_token(TokenKind::LParen) {
            let args = self.parse_attribute_args()?;
            self.expect(TokenKind::RParen)?;
            args
        } else {
            Vec::new()
        };

        let end = self.span();
        self.expect(TokenKind::RBracket)?;

        Ok(Attribute::new(name, args, start.merge(end)))
    }

    /// Parse attribute arguments: (arg1, arg2, key: value)
    fn parse_attribute_args(&mut self) -> Result<Vec<AttributeArg>> {
        let mut args = Vec::new();

        if self.check(TokenKind::RParen) {
            return Ok(args);
        }

        loop {
            // Check for named argument: identifier followed by colon
            if self.check(TokenKind::Identifier) && self.peek_ahead(1) == TokenKind::Colon {
                let name_token = self.advance().clone();
                let name = name_token.text;
                self.advance(); // skip colon
                let value = self.parse_expr()?;
                args.push(AttributeArg::Named(name, value));
            } else {
                // Positional argument
                let value = self.parse_expr()?;
                args.push(AttributeArg::Positional(value));
            }

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }

    // === Function parsing ===

    /// Parse function with optional attributes
    fn parse_function_with_attrs(&mut self, attributes: Attributes) -> Result<Function> {
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
            attributes,
            span: start.merge(end),
        })
    }

    #[allow(dead_code)]
    fn parse_function(&mut self) -> Result<Function> {
        self.parse_function_with_attrs(Attributes::new())
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
function main() {
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
        let source = "<?php function main() { $x: int = 1 + 2 * 3; }";
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn test_parse_arrow_closure() {
        let source = r#"<?php function main() { $f = fn($x: int): int => $x + 1; }"#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn test_parse_full_closure() {
        let source = r#"<?php
function main() {
    $y: int = 10;
    $f = function($x: int) use ($y): int {
        return $x + $y;
    };
}"#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn test_parse_closure_with_ref_capture() {
        let source = r#"<?php
function main() {
    $counter: int = 0;
    $inc = function() use (&$counter): void {
        $counter = $counter + 1;
    };
}"#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn test_parse_try_catch() {
        let source = r#"<?php
function main() {
    try {
        echo "try";
    } catch (Exception $e) {
        echo "catch";
    }
}"#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn test_parse_try_catch_finally() {
        let source = r#"<?php
function main() {
    try {
        echo "try";
    } catch (Exception $e) {
        echo "catch";
    } finally {
        echo "finally";
    }
}"#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn test_parse_multi_catch() {
        let source = r#"<?php
function main() {
    try {
        echo "risky";
    } catch (InvalidArgumentException|RuntimeException $e) {
        echo "caught";
    }
}"#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn test_parse_throw() {
        let source = r#"<?php
function main() {
    throw $e;
}"#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
    }
}
