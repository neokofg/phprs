//! Class and trait parsing

use crate::ast::{ClassDef, Method, Property, TraitDef, Type, Visibility};
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

        // Parse parent class (can be qualified name)
        let (parent, parent_qualified) = if self.match_token(TokenKind::Extends) {
            let qn = self.parse_qualified_name()?;
            let simple_name = qn.last().unwrap_or("").to_string();
            (Some(simple_name), Some(qn))
        } else {
            (None, None)
        };

        // Parse interfaces (can be qualified names)
        let mut interfaces = Vec::new();
        let mut interfaces_qualified = Vec::new();
        if self.match_token(TokenKind::Implements) {
            loop {
                let qn = self.parse_qualified_name()?;
                let simple_name = qn.last().unwrap_or("").to_string();
                interfaces.push(simple_name);
                interfaces_qualified.push(qn);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();
        let mut methods = Vec::new();
        let mut trait_uses = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            // Check for trait use: use SomeTrait;
            if self.check(TokenKind::Use) {
                trait_uses.push(self.parse_trait_use()?);
                continue;
            }

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
            qualified_name: None, // Will be set by resolver
            parent,
            parent_qualified,
            interfaces,
            interfaces_qualified,
            properties,
            methods,
            trait_uses,
            is_abstract,
            is_final,
            span: start.merge(end),
        })
    }

    /// Parse a trait definition
    pub(super) fn parse_trait(&mut self) -> Result<TraitDef> {
        let start = self.span();

        self.expect(TokenKind::Trait)?;

        let name_token = self.expect(TokenKind::Identifier)?;
        let name = name_token.text.clone();

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

        Ok(TraitDef {
            name,
            qualified_name: None, // Will be set by resolver
            properties,
            methods,
            span: start.merge(end),
        })
    }

    fn parse_class_member(&mut self) -> Result<(Option<Property>, Option<Method>)> {
        let start = self.span();

        // Parse modifiers in any order: abstract, final, static, visibility
        let mut visibility = Visibility::Public;
        let mut is_static = false;
        let mut is_abstract = false;
        let mut is_final = false;

        loop {
            match self.peek() {
                TokenKind::Public => {
                    self.advance();
                    visibility = Visibility::Public;
                }
                TokenKind::Private => {
                    self.advance();
                    visibility = Visibility::Private;
                }
                TokenKind::Protected => {
                    self.advance();
                    visibility = Visibility::Protected;
                }
                TokenKind::Static => {
                    self.advance();
                    is_static = true;
                }
                TokenKind::Abstract => {
                    self.advance();
                    is_abstract = true;
                }
                TokenKind::Final => {
                    self.advance();
                    is_final = true;
                }
                _ => break,
            }
        }

        // abstract and final are mutually exclusive
        if is_abstract && is_final {
            return Err(CompileError::ParserError {
                message: "Cannot use abstract and final together".to_string(),
                span: self.current().span,
            }
            .into());
        }

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
