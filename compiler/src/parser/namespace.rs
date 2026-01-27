//! Namespace and use declaration parsing

use crate::ast::{NamespaceDecl, QualifiedName, TraitUse, UseDecl, UseItem, UseKind};
use crate::errors::CompileError;
use crate::lexer::TokenKind;
use miette::Result;

use super::Parser;

impl Parser {
    /// Parse a qualified name: App\Models\User or \App\Models\User
    pub(super) fn parse_qualified_name(&mut self) -> Result<QualifiedName> {
        let start = self.span();

        // Check for leading backslash (absolute path)
        let is_absolute = self.match_token(TokenKind::Backslash);

        let mut segments = Vec::new();

        // First segment must be an identifier
        let first = self.expect(TokenKind::Identifier)?;
        segments.push(first.text.clone());

        // Parse remaining segments: \Segment
        while self.match_token(TokenKind::Backslash) {
            let segment = self.expect(TokenKind::Identifier)?;
            segments.push(segment.text.clone());
        }

        Ok(QualifiedName::new(
            segments,
            is_absolute,
            start.merge(self.span()),
        ))
    }

    /// Parse namespace declaration: namespace App\Models;
    pub(super) fn parse_namespace(&mut self) -> Result<NamespaceDecl> {
        let start = self.span();
        self.expect(TokenKind::Namespace)?;

        let name = self.parse_qualified_name()?;

        self.expect(TokenKind::Semicolon)?;

        Ok(NamespaceDecl {
            name,
            span: start.merge(self.span()),
        })
    }

    /// Parse use declaration: use App\User; or use App\User as U;
    /// Also supports: use function App\format; and use const App\DEBUG;
    pub(super) fn parse_use_declaration(&mut self) -> Result<UseDecl> {
        let start = self.span();
        self.expect(TokenKind::Use)?;

        // Check for use kind: function or const
        let kind = if self.check(TokenKind::Fn) {
            self.advance();
            UseKind::Function
        } else if self.check(TokenKind::Const) {
            self.advance();
            UseKind::Const
        } else {
            UseKind::Class
        };

        let mut items = Vec::new();

        loop {
            let item_start = self.span();
            let path = self.parse_qualified_name()?;

            // Check for alias: as Alias
            let alias = if self.match_token(TokenKind::As) {
                let alias_token = self.expect(TokenKind::Identifier)?;
                Some(alias_token.text.clone())
            } else {
                None
            };

            items.push(UseItem {
                path,
                alias,
                kind,
                span: item_start.merge(self.span()),
            });

            // Check for more items: use A, B, C;
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::Semicolon)?;

        Ok(UseDecl {
            items,
            span: start.merge(self.span()),
        })
    }

    /// Parse trait use inside a class: `use SomeTrait;`
    pub(super) fn parse_trait_use(&mut self) -> Result<TraitUse> {
        let start = self.span();
        self.expect(TokenKind::Use)?;

        let mut traits = Vec::new();

        loop {
            let trait_name = self.parse_qualified_name()?;
            traits.push(trait_name);

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::Semicolon)?;

        Ok(TraitUse {
            traits,
            span: start.merge(self.span()),
        })
    }

    /// Try to parse a type that might be a qualified name
    #[allow(dead_code)]
    pub(super) fn parse_type_with_qualified_name(
        &mut self,
    ) -> Result<(crate::ast::Type, Option<QualifiedName>)> {
        let is_ref = self.match_token(TokenKind::Ampersand);
        let is_nullable = self.match_token(TokenKind::Not);

        let (base_type, qualified) = match self.peek() {
            TokenKind::TypeInt => {
                self.advance();
                (crate::ast::Type::Int, None)
            }
            TokenKind::TypeFloat => {
                self.advance();
                (crate::ast::Type::Float, None)
            }
            TokenKind::TypeBool => {
                self.advance();
                (crate::ast::Type::Bool, None)
            }
            TokenKind::TypeString => {
                self.advance();
                (crate::ast::Type::String, None)
            }
            TokenKind::TypeVoid => {
                self.advance();
                (crate::ast::Type::Void, None)
            }
            TokenKind::SelfKw => {
                self.advance();
                (crate::ast::Type::SelfType, None)
            }
            TokenKind::Static => {
                self.advance();
                (crate::ast::Type::StaticType, None)
            }
            TokenKind::Backslash | TokenKind::Identifier => {
                // Could be a qualified name like \App\User or App\User
                let qn = self.parse_qualified_name()?;

                // Check for array<T> syntax
                if qn.is_simple() && qn.last() == Some("array") && self.check(TokenKind::Lt) {
                    self.advance();
                    let (inner, _) = self.parse_type_with_qualified_name()?;
                    self.expect(TokenKind::Gt)?;
                    (crate::ast::Type::Array(Box::new(inner)), None)
                } else {
                    // Class type - use the last segment as simple name for backwards compatibility
                    let simple_name = qn.last().unwrap_or("").to_string();
                    (crate::ast::Type::Class(simple_name), Some(qn))
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
            crate::ast::Type::Nullable(Box::new(base_type))
        } else {
            base_type
        };

        let final_type = if is_ref {
            crate::ast::Type::Ref(Box::new(typed))
        } else {
            typed
        };

        Ok((final_type, qualified))
    }
}
