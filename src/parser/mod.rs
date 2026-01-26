use crate::ast::{
    BinaryOp, Expr, ExprKind, Function, Param, Program, Span, Stmt, StmtKind, Type, UnaryOp,
};
use crate::errors::CompileError;
use crate::lexer::{SpannedToken, TokenKind};
use miette::Result;

pub fn parse(tokens: Vec<SpannedToken>) -> Result<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<SpannedToken>) -> Self {
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

    fn peek_ahead(&self, n: usize) -> TokenKind {
        self.tokens
            .get(self.pos + n)
            .map_or(TokenKind::Eof, |t| t.kind)
    }

    fn advance(&mut self) -> &SpannedToken {
        let _token = self.current();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        // Return reference to the token we just advanced past
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

        while !self.check(TokenKind::Eof) {
            functions.push(self.parse_function()?);
        }

        Ok(Program { functions })
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

        let return_type = if self.match_token(TokenKind::Arrow) {
            self.parse_type()?
        } else {
            Type::Void
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
        let name = name_token.text[1..].to_string(); // Remove $

        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;

        Ok(Param {
            name,
            ty,
            is_ref,
            span: start.merge(self.span()),
        })
    }

    fn parse_type(&mut self) -> Result<Type> {
        let is_ref = self.match_token(TokenKind::Ampersand);

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
            _ => {
                return Err(CompileError::ParserError {
                    message: format!("Expected type, found {:?}", self.peek()),
                    span: self.current().span,
                }
                .into());
            }
        };

        if is_ref {
            Ok(Type::Ref(Box::new(base_type)))
        } else {
            Ok(base_type)
        }
    }

    // === Statement parsing ===

    fn parse_block_contents(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek() {
            TokenKind::Return => self.parse_return(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Echo => self.parse_echo(),
            TokenKind::LBrace => self.parse_block(),
            TokenKind::Variable => self.parse_variable_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_return(&mut self) -> Result<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Return)?;

        let value = if !self.check(TokenKind::Semicolon) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::new(StmtKind::Return(value), start.merge(self.span())))
    }

    fn parse_if(&mut self) -> Result<Stmt> {
        let start = self.span();
        self.expect(TokenKind::If)?;
        self.expect(TokenKind::LParen)?;
        let condition = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;
        let then_branch = self.parse_block_contents()?;
        self.expect(TokenKind::RBrace)?;

        let else_branch = if self.match_token(TokenKind::Else) {
            if self.check(TokenKind::If) {
                // else if
                Some(vec![self.parse_if()?])
            } else {
                self.expect(TokenKind::LBrace)?;
                let stmts = self.parse_block_contents()?;
                self.expect(TokenKind::RBrace)?;
                Some(stmts)
            }
        } else {
            None
        };

        Ok(Stmt::new(
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            },
            start.merge(self.span()),
        ))
    }

    fn parse_while(&mut self) -> Result<Stmt> {
        let start = self.span();
        self.expect(TokenKind::While)?;
        self.expect(TokenKind::LParen)?;
        let condition = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block_contents()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Stmt::new(
            StmtKind::While { condition, body },
            start.merge(self.span()),
        ))
    }

    fn parse_for(&mut self) -> Result<Stmt> {
        let start = self.span();
        self.expect(TokenKind::For)?;
        self.expect(TokenKind::LParen)?;

        // Init
        let init = if !self.check(TokenKind::Semicolon) {
            Some(Box::new(self.parse_variable_stmt_no_semi()?))
        } else {
            None
        };
        self.expect(TokenKind::Semicolon)?;

        // Condition
        let condition = if !self.check(TokenKind::Semicolon) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::Semicolon)?;

        // Update
        let update = if !self.check(TokenKind::RParen) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block_contents()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Stmt::new(
            StmtKind::For {
                init,
                condition,
                update,
                body,
            },
            start.merge(self.span()),
        ))
    }

    fn parse_echo(&mut self) -> Result<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Echo)?;

        let mut exprs = vec![self.parse_expr()?];
        while self.match_token(TokenKind::Comma) {
            exprs.push(self.parse_expr()?);
        }

        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::new(StmtKind::Echo(exprs), start.merge(self.span())))
    }

    fn parse_block(&mut self) -> Result<Stmt> {
        let start = self.span();
        self.expect(TokenKind::LBrace)?;
        let stmts = self.parse_block_contents()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Stmt::new(StmtKind::Block(stmts), start.merge(self.span())))
    }

    fn parse_variable_stmt(&mut self) -> Result<Stmt> {
        let stmt = self.parse_variable_stmt_no_semi()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(stmt)
    }

    fn parse_variable_stmt_no_semi(&mut self) -> Result<Stmt> {
        let start = self.span();
        let name_token = self.expect(TokenKind::Variable)?;
        let name = name_token.text[1..].to_string();

        // Check for type annotation: $x: int = ...
        let ty = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Check for compound assignment
        match self.peek() {
            TokenKind::PlusAssign => {
                self.advance();
                let value = self.parse_expr()?;
                return Ok(Stmt::new(
                    StmtKind::CompoundAssign {
                        target: name,
                        op: BinaryOp::Add,
                        value,
                    },
                    start.merge(self.span()),
                ));
            }
            TokenKind::MinusAssign => {
                self.advance();
                let value = self.parse_expr()?;
                return Ok(Stmt::new(
                    StmtKind::CompoundAssign {
                        target: name,
                        op: BinaryOp::Sub,
                        value,
                    },
                    start.merge(self.span()),
                ));
            }
            TokenKind::StarAssign => {
                self.advance();
                let value = self.parse_expr()?;
                return Ok(Stmt::new(
                    StmtKind::CompoundAssign {
                        target: name,
                        op: BinaryOp::Mul,
                        value,
                    },
                    start.merge(self.span()),
                ));
            }
            TokenKind::SlashAssign => {
                self.advance();
                let value = self.parse_expr()?;
                return Ok(Stmt::new(
                    StmtKind::CompoundAssign {
                        target: name,
                        op: BinaryOp::Div,
                        value,
                    },
                    start.merge(self.span()),
                ));
            }
            _ => {}
        }

        self.expect(TokenKind::Assign)?;
        let init = self.parse_expr()?;

        if ty.is_some() {
            Ok(Stmt::new(
                StmtKind::Let { name, ty, init },
                start.merge(self.span()),
            ))
        } else {
            Ok(Stmt::new(
                StmtKind::Assign {
                    target: name,
                    value: init,
                },
                start.merge(self.span()),
            ))
        }
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt> {
        let start = self.span();
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::new(StmtKind::Expr(expr), start.merge(self.span())))
    }

    // === Expression parsing (Pratt parser) ===

    fn parse_expr(&mut self) -> Result<Expr> {
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
                _ => break,
            }
        }

        Ok(expr)
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
                // Remove quotes
                let s = &token.text[1..token.text.len() - 1];
                // Handle escape sequences
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
            TokenKind::Variable => {
                self.advance();
                let name = token.text[1..].to_string();

                // Check for assignment expression
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
            TokenKind::Identifier => {
                self.advance();
                let name = token.text.clone();

                // Function call
                if self.check(TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();

                    if !self.check(TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if !self.match_token(TokenKind::Comma) {
                                break;
                            }
                        }
                    }

                    self.expect(TokenKind::RParen)?;
                    return Ok(Expr::new(
                        ExprKind::Call { name, args },
                        start.merge(self.span()),
                    ));
                }

                Ok(Expr::new(ExprKind::Variable(name), start))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            _ => Err(CompileError::ParserError {
                message: format!("Unexpected token: {:?}", token.kind),
                span: token.span,
            }
            .into()),
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
