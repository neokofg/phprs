#![allow(
    clippy::too_many_lines,
    clippy::match_same_arms,
    clippy::missing_errors_doc
)]

use std::collections::HashMap;

use crate::ast::{Expr, ExprKind, Function, Program, Span, Stmt, StmtKind, Type};
use crate::errors::CompileError;
use miette::Result;

/// Check ownership rules for a program.
pub fn check(program: &Program) -> Result<()> {
    let mut checker = OwnershipChecker::new();
    checker.check_program(program)
}

/// State of a variable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VarState {
    /// Variable owns its value
    Owned,
    /// Value has been moved out
    Moved,
    /// Variable is borrowed immutably
    Borrowed,
    /// Variable is borrowed mutably
    BorrowedMut,
}

#[derive(Debug, Clone)]
struct VarInfo {
    state: VarState,
    ty: Type,
    def_span: Span,
    move_span: Option<Span>,
}

struct OwnershipChecker {
    /// Variable states in current scope
    scopes: Vec<HashMap<String, VarInfo>>,
    /// Active borrows
    borrows: Vec<(String, Span)>,
}

impl OwnershipChecker {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            borrows: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_var(&mut self, name: &str, ty: Type, span: Span) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.to_string(),
                VarInfo {
                    state: VarState::Owned,
                    ty,
                    def_span: span,
                    move_span: None,
                },
            );
        }
    }

    fn lookup_var(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    fn lookup_var_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                return Some(info);
            }
        }
        None
    }

    fn check_program(&mut self, program: &Program) -> Result<()> {
        for func in &program.functions {
            self.check_function(func)?;
        }
        Ok(())
    }

    fn check_function(&mut self, func: &Function) -> Result<()> {
        self.push_scope();

        // Add parameters to scope
        for param in &func.params {
            let state = if param.is_ref {
                VarState::Borrowed
            } else {
                VarState::Owned
            };

            if let Some(scope) = self.scopes.last_mut() {
                scope.insert(
                    param.name.clone(),
                    VarInfo {
                        state,
                        ty: param.ty.clone(),
                        def_span: param.span,
                        move_span: None,
                    },
                );
            }
        }

        // Check body
        for stmt in &func.body {
            self.check_stmt(stmt)?;
        }

        self.pop_scope();
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match &stmt.kind {
            StmtKind::Let { name, ty, init } => {
                // Check the initializer expression
                self.check_expr(init, true)?;

                // Define the new variable
                let var_ty = ty.clone().unwrap_or(Type::Unknown);
                self.define_var(name, var_ty, stmt.span);
            }

            StmtKind::Assign { target, value } => {
                // Check if target is accessible (not moved or borrowed mutably)
                if let Some(info) = self.lookup_var(target) {
                    if info.state == VarState::BorrowedMut {
                        return Err(CompileError::OwnershipError {
                            message: format!(
                                "Cannot assign to '{target}' while it is mutably borrowed"
                            ),
                            move_span: info.def_span.into(),
                            use_span: Some(stmt.span.into()),
                        }
                        .into());
                    }
                }

                self.check_expr(value, true)?;

                // Update variable state
                if let Some(info) = self.lookup_var_mut(target) {
                    info.state = VarState::Owned;
                    info.move_span = None;
                } else {
                    // New variable (implicit declaration)
                    let ty = value.ty.clone().unwrap_or(Type::Unknown);
                    self.define_var(target, ty, stmt.span);
                }
            }

            StmtKind::CompoundAssign { target, value, .. } => {
                self.check_var_use(target, stmt.span)?;
                self.check_expr(value, false)?;
            }

            StmtKind::Expr(expr) => {
                self.check_expr(expr, false)?;
            }

            StmtKind::Return(expr) => {
                if let Some(e) = expr {
                    self.check_expr(e, true)?;
                }
            }

            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.check_expr(condition, false)?;

                self.push_scope();
                for s in then_branch {
                    self.check_stmt(s)?;
                }
                self.pop_scope();

                if let Some(else_stmts) = else_branch {
                    self.push_scope();
                    for s in else_stmts {
                        self.check_stmt(s)?;
                    }
                    self.pop_scope();
                }
            }

            StmtKind::While { condition, body } => {
                self.check_expr(condition, false)?;

                self.push_scope();
                for s in body {
                    self.check_stmt(s)?;
                }
                self.pop_scope();
            }

            StmtKind::For {
                init,
                condition,
                update,
                body,
            } => {
                self.push_scope();

                if let Some(i) = init {
                    self.check_stmt(i)?;
                }
                if let Some(c) = condition {
                    self.check_expr(c, false)?;
                }
                if let Some(u) = update {
                    self.check_expr(u, false)?;
                }

                for s in body {
                    self.check_stmt(s)?;
                }

                self.pop_scope();
            }

            StmtKind::Echo(exprs) => {
                for e in exprs {
                    self.check_expr(e, false)?;
                }
            }

            StmtKind::Block(stmts) => {
                self.push_scope();
                for s in stmts {
                    self.check_stmt(s)?;
                }
                self.pop_scope();
            }
        }

        Ok(())
    }

    fn check_expr(&mut self, expr: &Expr, is_move_context: bool) -> Result<()> {
        match &expr.kind {
            ExprKind::IntLit(_)
            | ExprKind::FloatLit(_)
            | ExprKind::BoolLit(_)
            | ExprKind::StringLit(_)
            | ExprKind::Null => {}

            ExprKind::Variable(name) => {
                let info = self.lookup_var(name);

                if let Some(info) = info {
                    // Check if moved
                    if info.state == VarState::Moved {
                        return Err(CompileError::OwnershipError {
                            message: format!("Use of moved value: ${name}"),
                            move_span: info.move_span.unwrap_or(info.def_span).into(),
                            use_span: Some(expr.span.into()),
                        }
                        .into());
                    }

                    // Check if this is a move (non-Copy type in move context)
                    if is_move_context && !info.ty.is_copy() && info.state == VarState::Owned {
                        // Mark as moved
                        if let Some(info) = self.lookup_var_mut(name) {
                            info.state = VarState::Moved;
                            info.move_span = Some(expr.span);
                        }
                    }
                }
            }

            ExprKind::Binary { left, right, .. } => {
                self.check_expr(left, false)?;
                self.check_expr(right, false)?;
            }

            ExprKind::Unary { operand, .. } => {
                self.check_expr(operand, false)?;
            }

            ExprKind::Call { args, .. } => {
                // Function arguments are move contexts for non-ref params
                for arg in args {
                    self.check_expr(arg, true)?;
                }
            }

            ExprKind::Ref(inner) => {
                // Check the inner expression is valid
                if let ExprKind::Variable(name) = &inner.kind {
                    if let Some(info) = self.lookup_var(name) {
                        if info.state == VarState::Moved {
                            return Err(CompileError::OwnershipError {
                                message: format!("Cannot borrow moved value: ${name}"),
                                move_span: info.move_span.unwrap_or(info.def_span).into(),
                                use_span: Some(expr.span.into()),
                            }
                            .into());
                        }
                    }
                    self.borrows.push((name.clone(), expr.span));
                }
                self.check_expr(inner, false)?;
            }

            ExprKind::RefMut(inner) => {
                if let ExprKind::Variable(name) = &inner.kind {
                    if let Some(info) = self.lookup_var(name) {
                        if info.state == VarState::Moved {
                            return Err(CompileError::OwnershipError {
                                message: format!("Cannot mutably borrow moved value: ${name}"),
                                move_span: info.move_span.unwrap_or(info.def_span).into(),
                                use_span: Some(expr.span.into()),
                            }
                            .into());
                        }
                        if info.state == VarState::Borrowed || info.state == VarState::BorrowedMut {
                            return Err(CompileError::OwnershipError {
                                message: format!(
                                    "Cannot mutably borrow '{name}' while it is already borrowed"
                                ),
                                move_span: info.def_span.into(),
                                use_span: Some(expr.span.into()),
                            }
                            .into());
                        }
                    }
                    if let Some(info) = self.lookup_var_mut(name) {
                        info.state = VarState::BorrowedMut;
                    }
                }
                self.check_expr(inner, false)?;
            }

            ExprKind::Assign { target, value } => {
                self.check_var_use(target, expr.span)?;
                self.check_expr(value, true)?;
            }

            ExprKind::PrefixOp { target, .. } | ExprKind::PostfixOp { target, .. } => {
                self.check_var_use(target, expr.span)?;
            }

            // === OOP Expressions ===
            ExprKind::New { args, .. } => {
                for arg in args {
                    self.check_expr(arg, true)?;
                }
            }

            ExprKind::This => {
                // $this is always valid within a class context
            }

            ExprKind::PropertyAccess { object, .. } => {
                self.check_expr(object, false)?;
            }

            ExprKind::MethodCall { object, args, .. } => {
                self.check_expr(object, false)?;
                for arg in args {
                    self.check_expr(arg, true)?;
                }
            }

            ExprKind::StaticPropertyAccess { .. } => {
                // Static access doesn't involve ownership
            }

            ExprKind::StaticPropertyAssign { value, .. } => {
                self.check_expr(value, true)?;
            }

            ExprKind::StaticMethodCall { args, .. } => {
                for arg in args {
                    self.check_expr(arg, true)?;
                }
            }

            ExprKind::PropertyAssign { object, value, .. } => {
                self.check_expr(object, false)?;
                self.check_expr(value, true)?;
            }

            ExprKind::ArrayLit(elements) => {
                for elem in elements {
                    if let Some(key) = &elem.key {
                        self.check_expr(key, false)?;
                    }
                    self.check_expr(&elem.value, true)?;
                }
            }

            ExprKind::ArrayAccess { array, index } => {
                self.check_expr(array, false)?;
                self.check_expr(index, false)?;
            }
        }

        Ok(())
    }

    fn check_var_use(&self, name: &str, span: Span) -> Result<()> {
        if let Some(info) = self.lookup_var(name) {
            if info.state == VarState::Moved {
                return Err(CompileError::OwnershipError {
                    message: format!("Use of moved value: ${name}"),
                    move_span: info.move_span.unwrap_or(info.def_span).into(),
                    use_span: Some(span.into()),
                }
                .into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;
    use crate::types::check as type_check;

    fn check_source(source: &str) -> Result<()> {
        let tokens = tokenize(source)?;
        let ast = parse(tokens)?;
        let typed_ast = type_check(&ast)?;
        check(&typed_ast)
    }

    #[test]
    fn test_copy_type() {
        let source = r#"<?php
function main() {
    $x: int = 42;
    $y = $x;
    echo $x;  // OK - int is Copy
}
"#;
        assert!(check_source(source).is_ok());
    }

    #[test]
    fn test_move_string() {
        let source = r#"<?php
function main() {
    $s: string = "hello";
    $s2 = $s;
    echo $s;  // Error - string was moved
}
"#;
        assert!(check_source(source).is_err());
    }
}
