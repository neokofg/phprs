//! Expression parsing

use crate::ast::{ArrayElement, BinaryOp, Capture, ClosureBody, Expr, ExprKind, UnaryOp};
use crate::errors::CompileError;
use crate::lexer::TokenKind;
use miette::Result;

use super::Parser;

impl Parser {
    pub(super) fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                TokenKind::Eq => BinaryOp::Eq,
                TokenKind::Ne => BinaryOp::Ne,
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Le => BinaryOp::Le,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::Ge => BinaryOp::Ge,
                TokenKind::And => BinaryOp::And,
                TokenKind::Or => BinaryOp::Or,
                TokenKind::Dot => BinaryOp::Concat,
                _ => break,
            };

            let prec = op.precedence();
            if prec < min_bp {
                break;
            }

            self.advance();
            let right = self.parse_expr_bp(prec + 1)?;

            let span = left.span.merge(right.span);
            left = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        let start = self.span();

        match self.peek() {
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                let span = start.merge(operand.span);
                Ok(Expr::new(
                    ExprKind::Unary {
                        op: UnaryOp::Neg,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }
            TokenKind::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                let span = start.merge(operand.span);
                Ok(Expr::new(
                    ExprKind::Unary {
                        op: UnaryOp::Not,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }
            TokenKind::Ampersand => {
                self.advance();
                let operand = self.parse_unary()?;
                let span = start.merge(operand.span);
                Ok(Expr::new(ExprKind::Ref(Box::new(operand)), span))
            }
            TokenKind::PlusPlus => {
                self.advance();
                let var_token = self.expect(TokenKind::Variable)?;
                let name = var_token.text[1..].to_string();
                Ok(Expr::new(
                    ExprKind::PrefixOp {
                        op: UnaryOp::Inc,
                        target: name,
                    },
                    start.merge(self.span()),
                ))
            }
            TokenKind::MinusMinus => {
                self.advance();
                let var_token = self.expect(TokenKind::Variable)?;
                let name = var_token.text[1..].to_string();
                Ok(Expr::new(
                    ExprKind::PrefixOp {
                        op: UnaryOp::Dec,
                        target: name,
                    },
                    start.merge(self.span()),
                ))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                TokenKind::PlusPlus => {
                    self.advance();
                    if let ExprKind::Variable(name) = expr.kind {
                        expr = Expr::new(
                            ExprKind::PostfixOp {
                                op: UnaryOp::Inc,
                                target: name,
                            },
                            expr.span.merge(self.span()),
                        );
                    } else {
                        return Err(CompileError::ParserError {
                            message: "Expected variable for postfix operator".to_string(),
                            span: expr.span.into(),
                        }
                        .into());
                    }
                }
                TokenKind::MinusMinus => {
                    self.advance();
                    if let ExprKind::Variable(name) = expr.kind {
                        expr = Expr::new(
                            ExprKind::PostfixOp {
                                op: UnaryOp::Dec,
                                target: name,
                            },
                            expr.span.merge(self.span()),
                        );
                    } else {
                        return Err(CompileError::ParserError {
                            message: "Expected variable for postfix operator".to_string(),
                            span: expr.span.into(),
                        }
                        .into());
                    }
                }
                TokenKind::Arrow => {
                    expr = self.parse_member_access(expr)?;
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(TokenKind::RBracket)?;

                    expr = Expr::new(
                        ExprKind::ArrayAccess {
                            array: Box::new(expr),
                            index: Box::new(index),
                        },
                        self.span(),
                    );
                }
                TokenKind::LParen => {
                    // Closure call: $closure($args) or ($expr)($args)
                    if matches!(
                        expr.kind,
                        ExprKind::Variable(_)
                            | ExprKind::Closure { .. }
                            | ExprKind::PropertyAccess { .. }
                            | ExprKind::ArrayAccess { .. }
                    ) {
                        self.advance();
                        let args = self.parse_call_args()?;
                        self.expect(TokenKind::RParen)?;

                        expr = Expr::new(
                            ExprKind::ClosureCall {
                                closure: Box::new(expr),
                                args,
                            },
                            self.span(),
                        );
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_member_access(&mut self, object: Expr) -> Result<Expr> {
        self.advance(); // consume ->
        let member_token = self.expect(TokenKind::Identifier)?;
        let member = member_token.text.clone();

        if self.check(TokenKind::LParen) {
            // Method call
            self.advance();
            let args = self.parse_call_args()?;
            self.expect(TokenKind::RParen)?;

            Ok(Expr::new(
                ExprKind::MethodCall {
                    object: Box::new(object),
                    method: member,
                    args,
                },
                self.span(),
            ))
        } else if self.check(TokenKind::Assign) {
            // Property assignment
            self.advance();
            let value = self.parse_expr()?;

            Ok(Expr::new(
                ExprKind::PropertyAssign {
                    object: Box::new(object),
                    property: member,
                    value: Box::new(value),
                },
                self.span(),
            ))
        } else {
            // Property access
            Ok(Expr::new(
                ExprKind::PropertyAccess {
                    object: Box::new(object),
                    property: member,
                },
                self.span(),
            ))
        }
    }

    pub(super) fn parse_call_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                args.push(self.parse_expr()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let start = self.span();
        let token = self.current().clone();

        match token.kind {
            TokenKind::Integer => {
                self.advance();
                let value: i64 = token.text.parse().map_err(|_| CompileError::ParserError {
                    message: "Invalid integer literal".to_string(),
                    span: token.span,
                })?;
                Ok(Expr::new(ExprKind::IntLit(value), start))
            }
            TokenKind::Float => {
                self.advance();
                let value: f64 = token.text.parse().map_err(|_| CompileError::ParserError {
                    message: "Invalid float literal".to_string(),
                    span: token.span,
                })?;
                Ok(Expr::new(ExprKind::FloatLit(value), start))
            }
            TokenKind::String | TokenKind::StringSingle => {
                self.advance();
                let s = &token.text[1..token.text.len() - 1];
                let s = s
                    .replace("\\n", "\n")
                    .replace("\\t", "\t")
                    .replace("\\\"", "\"");
                Ok(Expr::new(ExprKind::StringLit(s), start))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::new(ExprKind::BoolLit(true), start))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::new(ExprKind::BoolLit(false), start))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expr::new(ExprKind::Null, start))
            }
            TokenKind::This => {
                self.advance();
                Ok(Expr::new(ExprKind::This, start))
            }
            TokenKind::Parent => {
                // parent::method() call
                self.advance();
                self.expect(TokenKind::DoubleColon)?;

                let method_token = self.expect(TokenKind::Identifier)?;
                let method = method_token.text.clone();

                self.expect(TokenKind::LParen)?;
                let args = self.parse_call_args()?;
                self.expect(TokenKind::RParen)?;

                Ok(Expr::new(
                    ExprKind::StaticMethodCall {
                        class_name: "parent".to_string(),
                        method,
                        args,
                    },
                    start.merge(self.span()),
                ))
            }
            TokenKind::Variable => self.parse_variable_expr(start, &token.text),
            TokenKind::New => self.parse_new_expr(start),
            TokenKind::Identifier => self.parse_identifier_expr(start, &token.text),
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBracket => self.parse_array_lit(start),
            // Short closure: fn($x) => $x + 1
            TokenKind::FnArrow => self.parse_arrow_closure(start),
            // Full closure: function($x) use ($y) { return $x + $y; }
            TokenKind::Fn => self.parse_full_closure(start),
            _ => Err(CompileError::ParserError {
                message: format!("Unexpected token: {:?}", token.kind),
                span: token.span,
            }
            .into()),
        }
    }

    fn parse_variable_expr(&mut self, start: crate::ast::Span, text: &str) -> Result<Expr> {
        self.advance();
        let name = text[1..].to_string();

        if self.check(TokenKind::Assign) {
            self.advance();
            let value = self.parse_expr()?;
            return Ok(Expr::new(
                ExprKind::Assign {
                    target: name,
                    value: Box::new(value),
                },
                start.merge(self.span()),
            ));
        }

        Ok(Expr::new(ExprKind::Variable(name), start))
    }

    fn parse_new_expr(&mut self, start: crate::ast::Span) -> Result<Expr> {
        self.advance();
        let class_token = self.expect(TokenKind::Identifier)?;
        let class_name = class_token.text.clone();

        self.expect(TokenKind::LParen)?;
        let args = self.parse_call_args()?;
        self.expect(TokenKind::RParen)?;

        Ok(Expr::new(
            ExprKind::New { class_name, args },
            start.merge(self.span()),
        ))
    }

    fn parse_identifier_expr(&mut self, start: crate::ast::Span, text: &str) -> Result<Expr> {
        self.advance();
        let name = text.to_string();

        // Static access
        if self.check(TokenKind::DoubleColon) {
            self.advance();

            if self.check(TokenKind::Variable) {
                let var_token = self.advance().clone();
                let property = var_token.text[1..].to_string();

                // Check for assignment
                if self.check(TokenKind::Assign) {
                    self.advance();
                    let value = self.parse_expr()?;
                    return Ok(Expr::new(
                        ExprKind::StaticPropertyAssign {
                            class_name: name,
                            property,
                            value: Box::new(value),
                        },
                        start.merge(self.span()),
                    ));
                }

                return Ok(Expr::new(
                    ExprKind::StaticPropertyAccess {
                        class_name: name,
                        property,
                    },
                    start.merge(self.span()),
                ));
            }

            let method_token = self.expect(TokenKind::Identifier)?;
            let method = method_token.text.clone();

            self.expect(TokenKind::LParen)?;
            let args = self.parse_call_args()?;
            self.expect(TokenKind::RParen)?;

            return Ok(Expr::new(
                ExprKind::StaticMethodCall {
                    class_name: name,
                    method,
                    args,
                },
                start.merge(self.span()),
            ));
        }

        // Function call
        if self.check(TokenKind::LParen) {
            self.advance();
            let args = self.parse_call_args()?;
            self.expect(TokenKind::RParen)?;
            return Ok(Expr::new(
                ExprKind::Call { name, args },
                start.merge(self.span()),
            ));
        }

        Ok(Expr::new(ExprKind::Variable(name), start))
    }

    fn parse_array_lit(&mut self, start: crate::ast::Span) -> Result<Expr> {
        self.advance();
        let mut elements = Vec::new();

        if !self.check(TokenKind::RBracket) {
            loop {
                let first_expr = self.parse_expr()?;

                if self.match_token(TokenKind::FatArrow) {
                    let value = self.parse_expr()?;
                    elements.push(ArrayElement {
                        key: Some(first_expr),
                        value,
                    });
                } else {
                    elements.push(ArrayElement {
                        key: None,
                        value: first_expr,
                    });
                }

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
                if self.check(TokenKind::RBracket) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RBracket)?;
        Ok(Expr::new(
            ExprKind::ArrayLit(elements),
            start.merge(self.span()),
        ))
    }

    /// Parse arrow closure: fn($x) => $x + 1
    fn parse_arrow_closure(&mut self, start: crate::ast::Span) -> Result<Expr> {
        self.advance(); // consume 'fn'

        // Check for static
        let is_static = self.match_token(TokenKind::Static);

        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;

        // Optional return type
        let return_type = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::FatArrow)?;
        let body_expr = self.parse_expr()?;
        let end = body_expr.span;

        Ok(Expr::new(
            ExprKind::Closure {
                params,
                return_type,
                body: ClosureBody::Arrow(Box::new(body_expr)),
                captures: Vec::new(), // Arrow closures auto-capture
                is_static,
            },
            start.merge(end),
        ))
    }

    /// Parse full closure: function($x) use ($y, &$z) { return $x + $y; }
    fn parse_full_closure(&mut self, start: crate::ast::Span) -> Result<Expr> {
        self.advance(); // consume 'function'

        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;

        // Parse use clause: use ($x, &$y)
        let captures = if self.match_token(TokenKind::Use) {
            self.parse_captures()?
        } else {
            Vec::new()
        };

        // Optional return type
        let return_type = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block_contents()?;
        let end = self.span();
        self.expect(TokenKind::RBrace)?;

        Ok(Expr::new(
            ExprKind::Closure {
                params,
                return_type,
                body: ClosureBody::Block(body),
                captures,
                is_static: false,
            },
            start.merge(end),
        ))
    }

    /// Parse captures: ($x, &$y)
    fn parse_captures(&mut self) -> Result<Vec<Capture>> {
        self.expect(TokenKind::LParen)?;
        let mut captures = Vec::new();

        if !self.check(TokenKind::RParen) {
            loop {
                let start = self.span();
                let by_ref = self.match_token(TokenKind::Ampersand);

                let var_token = self.expect(TokenKind::Variable)?;
                let name = var_token.text[1..].to_string();

                captures.push(Capture {
                    name,
                    by_ref,
                    span: start.merge(self.span()),
                });

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RParen)?;
        Ok(captures)
    }
}
