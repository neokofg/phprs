//! Statement type checking

#![allow(clippy::ref_option)]

use crate::ast::{Stmt, StmtKind, Type};
use crate::errors::CompileError;
use miette::Result;

use super::TypeChecker;

impl TypeChecker {
    pub(super) fn check_stmt(&mut self, stmt: &Stmt, return_type: &Type) -> Result<Stmt> {
        let kind = match &stmt.kind {
            StmtKind::Let { name, ty, init } => {
                self.check_let_stmt(name, ty, init, stmt)?
            }
            StmtKind::Assign { target, value } => {
                self.check_assign_stmt(target, value, stmt)?
            }
            StmtKind::CompoundAssign { target, op, value } => {
                let value_typed = self.check_expr(value)?;
                StmtKind::CompoundAssign {
                    target: target.clone(),
                    op: *op,
                    value: value_typed,
                }
            }
            StmtKind::Expr(expr) => {
                let typed = self.check_expr(expr)?;
                StmtKind::Expr(typed)
            }
            StmtKind::Return(expr) => {
                self.check_return_stmt(expr, return_type, stmt)?
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.check_if_stmt(condition, then_branch, else_branch, return_type)?
            }
            StmtKind::While { condition, body } => {
                self.check_while_stmt(condition, body, return_type)?
            }
            StmtKind::For {
                init,
                condition,
                update,
                body,
            } => {
                self.check_for_stmt(init, condition, update, body, return_type)?
            }
            StmtKind::Echo(exprs) => {
                let mut typed = Vec::new();
                for e in exprs {
                    typed.push(self.check_expr(e)?);
                }
                StmtKind::Echo(typed)
            }
            StmtKind::Block(stmts) => {
                self.push_scope();
                let mut typed = Vec::new();
                for s in stmts {
                    typed.push(self.check_stmt(s, return_type)?);
                }
                self.pop_scope();
                StmtKind::Block(typed)
            }
        };

        Ok(Stmt::new(kind, stmt.span))
    }

    fn check_let_stmt(
        &mut self,
        name: &str,
        ty: &Option<Type>,
        init: &crate::ast::Expr,
        stmt: &Stmt,
    ) -> Result<StmtKind> {
        let init_typed = self.check_expr(init)?;
        let init_type = init_typed.ty.clone().unwrap_or(Type::Unknown);

        let var_type = if let Some(declared_ty) = ty {
            if !self.types_compatible(declared_ty, &init_type) {
                return Err(CompileError::TypeError {
                    message: format!(
                        "Type mismatch: expected {declared_ty}, found {init_type}"
                    ),
                    span: stmt.span.into(),
                }
                .into());
            }
            declared_ty.clone()
        } else {
            init_type
        };

        self.define_var(name, var_type.clone());

        Ok(StmtKind::Let {
            name: name.to_string(),
            ty: Some(var_type),
            init: init_typed,
        })
    }

    fn check_assign_stmt(
        &mut self,
        target: &str,
        value: &crate::ast::Expr,
        stmt: &Stmt,
    ) -> Result<StmtKind> {
        let value_typed = self.check_expr(value)?;

        if let Some(var_type) = self.lookup_var(target).cloned() {
            let value_type = value_typed.ty.clone().unwrap_or(Type::Unknown);
            if !self.types_compatible(&var_type, &value_type) {
                return Err(CompileError::TypeError {
                    message: format!(
                        "Type mismatch: variable {target} has type {var_type}, found {value_type}"
                    ),
                    span: stmt.span.into(),
                }
                .into());
            }
        } else {
            let value_type = value_typed.ty.clone().unwrap_or(Type::Unknown);
            self.define_var(target, value_type);
        }

        Ok(StmtKind::Assign {
            target: target.to_string(),
            value: value_typed,
        })
    }

    fn check_return_stmt(
        &mut self,
        expr: &Option<crate::ast::Expr>,
        return_type: &Type,
        stmt: &Stmt,
    ) -> Result<StmtKind> {
        if let Some(e) = expr {
            let typed = self.check_expr(e)?;
            let expr_type = typed.ty.clone().unwrap_or(Type::Unknown);

            if !self.types_compatible(return_type, &expr_type) {
                return Err(CompileError::TypeError {
                    message: format!(
                        "Return type mismatch: expected {return_type}, found {expr_type}"
                    ),
                    span: stmt.span.into(),
                }
                .into());
            }

            Ok(StmtKind::Return(Some(typed)))
        } else {
            if *return_type != Type::Void {
                return Err(CompileError::TypeError {
                    message: format!(
                        "Return type mismatch: expected {return_type}, found void"
                    ),
                    span: stmt.span.into(),
                }
                .into());
            }
            Ok(StmtKind::Return(None))
        }
    }

    fn check_if_stmt(
        &mut self,
        condition: &crate::ast::Expr,
        then_branch: &[Stmt],
        else_branch: &Option<Vec<Stmt>>,
        return_type: &Type,
    ) -> Result<StmtKind> {
        let cond_typed = self.check_expr(condition)?;
        let cond_type = cond_typed.ty.clone().unwrap_or(Type::Unknown);

        if !self.types_compatible(&Type::Bool, &cond_type) {
            return Err(CompileError::TypeError {
                message: format!("If condition must be bool, found {cond_type}"),
                span: condition.span.into(),
            }
            .into());
        }

        self.push_scope();
        let mut then_typed = Vec::new();
        for s in then_branch {
            then_typed.push(self.check_stmt(s, return_type)?);
        }
        self.pop_scope();

        let else_typed = if let Some(else_stmts) = else_branch {
            self.push_scope();
            let mut typed = Vec::new();
            for s in else_stmts {
                typed.push(self.check_stmt(s, return_type)?);
            }
            self.pop_scope();
            Some(typed)
        } else {
            None
        };

        Ok(StmtKind::If {
            condition: cond_typed,
            then_branch: then_typed,
            else_branch: else_typed,
        })
    }

    fn check_while_stmt(
        &mut self,
        condition: &crate::ast::Expr,
        body: &[Stmt],
        return_type: &Type,
    ) -> Result<StmtKind> {
        let cond_typed = self.check_expr(condition)?;

        self.push_scope();
        let mut body_typed = Vec::new();
        for s in body {
            body_typed.push(self.check_stmt(s, return_type)?);
        }
        self.pop_scope();

        Ok(StmtKind::While {
            condition: cond_typed,
            body: body_typed,
        })
    }

    fn check_for_stmt(
        &mut self,
        init: &Option<Box<Stmt>>,
        condition: &Option<crate::ast::Expr>,
        update: &Option<crate::ast::Expr>,
        body: &[Stmt],
        return_type: &Type,
    ) -> Result<StmtKind> {
        self.push_scope();

        let init_typed = if let Some(i) = init {
            Some(Box::new(self.check_stmt(i, return_type)?))
        } else {
            None
        };

        let cond_typed = if let Some(c) = condition {
            Some(self.check_expr(c)?)
        } else {
            None
        };

        let update_typed = if let Some(u) = update {
            Some(self.check_expr(u)?)
        } else {
            None
        };

        let mut body_typed = Vec::new();
        for s in body {
            body_typed.push(self.check_stmt(s, return_type)?);
        }

        self.pop_scope();

        Ok(StmtKind::For {
            init: init_typed,
            condition: cond_typed,
            update: update_typed,
            body: body_typed,
        })
    }
}
