//! Statement parsing

use crate::ast::{BinaryOp, CatchClause, Stmt, StmtKind};
use crate::errors::CompileError;
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
            TokenKind::Try => self.parse_try_catch(),
            TokenKind::Throw => self.parse_throw(),
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

        // Check if this is a property access ($var->...) or array access ($var[...])
        // If so, parse as expression statement
        if self.peek_ahead(1) == TokenKind::Arrow || self.peek_ahead(1) == TokenKind::LBracket {
            let expr = self.parse_expr()?;
            return Ok(Stmt::new(StmtKind::Expr(expr), start.merge(self.span())));
        }

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

    /// Parse try-catch-finally: try { ... } catch (Exception $e) { ... } finally { ... }
    fn parse_try_catch(&mut self) -> Result<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Try)?;

        // Parse try block
        self.expect(TokenKind::LBrace)?;
        let try_block = self.parse_block_contents()?;
        self.expect(TokenKind::RBrace)?;

        // Parse catch clauses
        let mut catches = Vec::new();
        while self.check(TokenKind::Catch) {
            catches.push(self.parse_catch_clause()?);
        }

        // Parse optional finally block
        let finally_block = if self.match_token(TokenKind::Finally) {
            self.expect(TokenKind::LBrace)?;
            let block = self.parse_block_contents()?;
            self.expect(TokenKind::RBrace)?;
            Some(block)
        } else {
            None
        };

        // Must have at least one catch or a finally
        if catches.is_empty() && finally_block.is_none() {
            return Err(CompileError::ParserError {
                message: "try block must have at least one catch or finally clause".to_string(),
                span: self.current().span,
            }
            .into());
        }

        Ok(Stmt::new(
            StmtKind::TryCatch {
                try_block,
                catches,
                finally_block,
            },
            start.merge(self.span()),
        ))
    }

    /// Parse catch clause: catch (Exception $e) { ... } or catch (Exception|Error $e) { ... }
    fn parse_catch_clause(&mut self) -> Result<CatchClause> {
        let start = self.span();
        self.expect(TokenKind::Catch)?;
        self.expect(TokenKind::LParen)?;

        // Parse exception types (can be multiple with |)
        let mut exception_types = Vec::new();
        loop {
            let type_token = self.expect(TokenKind::Identifier)?;
            exception_types.push(type_token.text.clone());

            if !self.match_token(TokenKind::Pipe) {
                break;
            }
        }

        // Parse variable
        let var_token = self.expect(TokenKind::Variable)?;
        let variable = var_token.text[1..].to_string();

        self.expect(TokenKind::RParen)?;

        // Parse catch body
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block_contents()?;
        self.expect(TokenKind::RBrace)?;

        Ok(CatchClause {
            exception_types,
            variable,
            body,
            span: start.merge(self.span()),
        })
    }

    /// Parse throw statement: throw $exception;
    fn parse_throw(&mut self) -> Result<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Throw)?;

        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::new(StmtKind::Throw(expr), start.merge(self.span())))
    }
}
