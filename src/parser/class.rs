//! Class parsing

use crate::ast::{ClassDef, Method, Property, Type, Visibility};
use crate::errors::CompileError;
use crate::lexer::TokenKind;
use miette::Result;

use super::Parser;

impl Parser {
    pub(super) fn parse_class(&mut self) -> Result<ClassDef> {
        let start = self.span();

        let is_abstract = self.match_token(TokenKind::Abstract);
        let is_final = if is_abstract {
            false
        } else {
            self.match_token(TokenKind::Final)
        };

        self.expect(TokenKind::Class)?;

        let name_token = self.expect(TokenKind::Identifier)?;
        let name = name_token.text.clone();

        let parent = if self.match_token(TokenKind::Extends) {
            let parent_token = self.expect(TokenKind::Identifier)?;
            Some(parent_token.text.clone())
        } else {
            None
        };

        let mut interfaces = Vec::new();
        if self.match_token(TokenKind::Implements) {
            loop {
                let iface_token = self.expect(TokenKind::Identifier)?;
                interfaces.push(iface_token.text.clone());
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();
        let mut methods = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let (prop, meth) = self.parse_class_member()?;
            if let Some(p) = prop {
                properties.push(p);
            }
            if let Some(m) = meth {
                methods.push(m);
            }
        }

        let end = self.span();
        self.expect(TokenKind::RBrace)?;

        Ok(ClassDef {
            name,
            parent,
            interfaces,
            properties,
            methods,
            is_abstract,
            is_final,
            span: start.merge(end),
        })
    }

    fn parse_class_member(&mut self) -> Result<(Option<Property>, Option<Method>)> {
        let start = self.span();

        let visibility = match self.peek() {
            TokenKind::Public => {
                self.advance();
                Visibility::Public
            }
            TokenKind::Private => {
                self.advance();
                Visibility::Private
            }
            TokenKind::Protected => {
                self.advance();
                Visibility::Protected
            }
            _ => Visibility::Public,
        };

        let is_static = self.match_token(TokenKind::Static);
        let is_abstract = self.match_token(TokenKind::Abstract);
        let is_final = if is_abstract {
            false
        } else {
            self.match_token(TokenKind::Final)
        };

        if self.check(TokenKind::Fn) {
            self.parse_method(start, visibility, is_static, is_abstract, is_final)
        } else if self.check(TokenKind::Variable) {
            self.parse_property(start, visibility, is_static)
        } else {
            Err(CompileError::ParserError {
                message: format!("Expected property or method, found {:?}", self.peek()),
                span: self.current().span,
            }
            .into())
        }
    }

    fn parse_method(
        &mut self,
        start: crate::ast::Span,
        visibility: Visibility,
        is_static: bool,
        is_abstract: bool,
        is_final: bool,
    ) -> Result<(Option<Property>, Option<Method>)> {
        self.advance(); // consume 'fn'
        let name_token = self.expect(TokenKind::Identifier)?;
        let name = name_token.text.clone();

        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;

        let return_type = if self.match_token(TokenKind::Colon) {
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = if is_abstract || self.check(TokenKind::Semicolon) {
            if self.check(TokenKind::Semicolon) {
                self.advance();
            }
            None
        } else {
            self.expect(TokenKind::LBrace)?;
            let stmts = self.parse_block_contents()?;
            self.expect(TokenKind::RBrace)?;
            Some(stmts)
        };

        Ok((
            None,
            Some(Method {
                name,
                params,
                return_type,
                visibility,
                is_static,
                is_abstract,
                is_final,
                body,
                span: start.merge(self.span()),
            }),
        ))
    }

    fn parse_property(
        &mut self,
        start: crate::ast::Span,
        visibility: Visibility,
        is_static: bool,
    ) -> Result<(Option<Property>, Option<Method>)> {
        let var_token = self.expect(TokenKind::Variable)?;
        let name = var_token.text[1..].to_string();

        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;

        let default = if self.match_token(TokenKind::Assign) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok((
            Some(Property {
                name,
                ty,
                visibility,
                is_static,
                default,
                span: start.merge(self.span()),
            }),
            None,
        ))
    }
}
