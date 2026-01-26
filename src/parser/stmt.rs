//! Statement parsing

use crate::ast::{BinaryOp, Stmt, StmtKind};
use crate::lexer::TokenKind;
use miette::Result;

use super::Parser;

impl Parser {
    pub(super) fn parse_block_contents(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    pub(super) fn parse_stmt(&mut self) -> Result<Stmt> {
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

        let value = if self.check(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
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

        let init = if self.check(TokenKind::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_variable_stmt_no_semi()?))
        };
        self.expect(TokenKind::Semicolon)?;

        let condition = if self.check(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(TokenKind::Semicolon)?;

        let update = if self.check(TokenKind::RParen) {
            None
        } else {
            Some(self.parse_expr()?)
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

    pub(super) fn parse_variable_stmt(&mut self) -> Result<Stmt> {
        let stmt = self.parse_variable_stmt_no_semi()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(stmt)
    }

    pub(super) fn parse_variable_stmt_no_semi(&mut self) -> Result<Stmt> {
        let start = self.span();
        let name_token = self.expect(TokenKind::Variable)?;
        let name = name_token.text[1..].to_string();

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
}
