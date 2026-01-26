use std::collections::HashMap;

use crate::ast::{
    BinaryOp, Expr, ExprKind, Function, Program, Span, Stmt, StmtKind, Type, UnaryOp,
};
use crate::errors::CompileError;
use miette::Result;

pub fn check(program: &Program) -> Result<Program> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}

struct TypeChecker {
    /// Function signatures: name -> (params, return_type)
    functions: HashMap<String, (Vec<Type>, Type)>,
    /// Variable types in current scope
    variables: Vec<HashMap<String, Type>>,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            variables: vec![HashMap::new()],
        }
    }

    fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.variables.pop();
    }

    fn define_var(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup_var(&self, name: &str) -> Option<&Type> {
        for scope in self.variables.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    fn check_program(&mut self, program: &Program) -> Result<Program> {
        // First pass: collect function signatures
        for func in &program.functions {
            let param_types: Vec<Type> = func.params.iter().map(|p| p.ty.clone()).collect();
            self.functions
                .insert(func.name.clone(), (param_types, func.return_type.clone()));
        }

        // Second pass: type check function bodies
        let mut typed_functions = Vec::new();
        for func in &program.functions {
            typed_functions.push(self.check_function(func)?);
        }

        Ok(Program {
            functions: typed_functions,
        })
    }

    fn check_function(&mut self, func: &Function) -> Result<Function> {
        self.push_scope();

        // Add parameters to scope
        for param in &func.params {
            self.define_var(&param.name, param.ty.clone());
        }

        // Type check body
        let mut typed_body = Vec::new();
        for stmt in &func.body {
            typed_body.push(self.check_stmt(stmt, &func.return_type)?);
        }

        self.pop_scope();

        Ok(Function {
            name: func.name.clone(),
            params: func.params.clone(),
            return_type: func.return_type.clone(),
            body: typed_body,
            span: func.span,
        })
    }

    fn check_stmt(&mut self, stmt: &Stmt, return_type: &Type) -> Result<Stmt> {
        let kind = match &stmt.kind {
            StmtKind::Let { name, ty, init } => {
                let init_typed = self.check_expr(init)?;
                let init_type = init_typed.ty.clone().unwrap_or(Type::Unknown);

                let var_type = if let Some(declared_ty) = ty {
                    // Check type compatibility
                    if !self.types_compatible(declared_ty, &init_type) {
                        return Err(CompileError::TypeError {
                            message: format!(
                                "Type mismatch: expected {}, found {}",
                                declared_ty, init_type
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

                StmtKind::Let {
                    name: name.clone(),
                    ty: Some(var_type),
                    init: init_typed,
                }
            }

            StmtKind::Assign { target, value } => {
                let value_typed = self.check_expr(value)?;

                if let Some(var_type) = self.lookup_var(target).cloned() {
                    let value_type = value_typed.ty.clone().unwrap_or(Type::Unknown);
                    if !self.types_compatible(&var_type, &value_type) {
                        return Err(CompileError::TypeError {
                            message: format!(
                                "Type mismatch: variable {} has type {}, found {}",
                                target, var_type, value_type
                            ),
                            span: stmt.span.into(),
                        }
                        .into());
                    }
                } else {
                    // First assignment - infer type
                    let value_type = value_typed.ty.clone().unwrap_or(Type::Unknown);
                    self.define_var(target, value_type);
                }

                StmtKind::Assign {
                    target: target.clone(),
                    value: value_typed,
                }
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
                if let Some(e) = expr {
                    let typed = self.check_expr(e)?;
                    let expr_type = typed.ty.clone().unwrap_or(Type::Unknown);

                    if !self.types_compatible(return_type, &expr_type) {
                        return Err(CompileError::TypeError {
                            message: format!(
                                "Return type mismatch: expected {}, found {}",
                                return_type, expr_type
                            ),
                            span: stmt.span.into(),
                        }
                        .into());
                    }

                    StmtKind::Return(Some(typed))
                } else {
                    if *return_type != Type::Void {
                        return Err(CompileError::TypeError {
                            message: format!(
                                "Return type mismatch: expected {}, found void",
                                return_type
                            ),
                            span: stmt.span.into(),
                        }
                        .into());
                    }
                    StmtKind::Return(None)
                }
            }

            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_typed = self.check_expr(condition)?;
                let cond_type = cond_typed.ty.clone().unwrap_or(Type::Unknown);

                if !self.types_compatible(&Type::Bool, &cond_type) {
                    return Err(CompileError::TypeError {
                        message: format!("If condition must be bool, found {}", cond_type),
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

                StmtKind::If {
                    condition: cond_typed,
                    then_branch: then_typed,
                    else_branch: else_typed,
                }
            }

            StmtKind::While { condition, body } => {
                let cond_typed = self.check_expr(condition)?;

                self.push_scope();
                let mut body_typed = Vec::new();
                for s in body {
                    body_typed.push(self.check_stmt(s, return_type)?);
                }
                self.pop_scope();

                StmtKind::While {
                    condition: cond_typed,
                    body: body_typed,
                }
            }

            StmtKind::For {
                init,
                condition,
                update,
                body,
            } => {
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

                StmtKind::For {
                    init: init_typed,
                    condition: cond_typed,
                    update: update_typed,
                    body: body_typed,
                }
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

    fn check_expr(&mut self, expr: &Expr) -> Result<Expr> {
        let (kind, ty) = match &expr.kind {
            ExprKind::IntLit(v) => (ExprKind::IntLit(*v), Type::Int),
            ExprKind::FloatLit(v) => (ExprKind::FloatLit(*v), Type::Float),
            ExprKind::BoolLit(v) => (ExprKind::BoolLit(*v), Type::Bool),
            ExprKind::StringLit(s) => (ExprKind::StringLit(s.clone()), Type::String),
            ExprKind::Null => (ExprKind::Null, Type::Unknown),

            ExprKind::Variable(name) => {
                let ty = self.lookup_var(name).cloned().unwrap_or(Type::Unknown);
                (ExprKind::Variable(name.clone()), ty)
            }

            ExprKind::Binary { left, op, right } => {
                let left_typed = self.check_expr(left)?;
                let right_typed = self.check_expr(right)?;

                let left_ty = left_typed.ty.clone().unwrap_or(Type::Unknown);
                let right_ty = right_typed.ty.clone().unwrap_or(Type::Unknown);

                let result_ty = self.binary_result_type(*op, &left_ty, &right_ty, expr.span)?;

                (
                    ExprKind::Binary {
                        left: Box::new(left_typed),
                        op: *op,
                        right: Box::new(right_typed),
                    },
                    result_ty,
                )
            }

            ExprKind::Unary { op, operand } => {
                let operand_typed = self.check_expr(operand)?;
                let operand_ty = operand_typed.ty.clone().unwrap_or(Type::Unknown);

                let result_ty = match op {
                    UnaryOp::Neg => {
                        if matches!(operand_ty, Type::Int | Type::Float) {
                            operand_ty
                        } else {
                            return Err(CompileError::TypeError {
                                message: format!("Cannot negate type {}", operand_ty),
                                span: expr.span.into(),
                            }
                            .into());
                        }
                    }
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::Inc | UnaryOp::Dec => operand_ty,
                };

                (
                    ExprKind::Unary {
                        op: *op,
                        operand: Box::new(operand_typed),
                    },
                    result_ty,
                )
            }

            ExprKind::Call { name, args } => {
                let (param_types, return_ty) =
                    self.functions
                        .get(name)
                        .cloned()
                        .ok_or_else(|| CompileError::TypeError {
                            message: format!("Unknown function: {}", name),
                            span: expr.span.into(),
                        })?;

                if args.len() != param_types.len() {
                    return Err(CompileError::TypeError {
                        message: format!(
                            "Function {} expects {} arguments, got {}",
                            name,
                            param_types.len(),
                            args.len()
                        ),
                        span: expr.span.into(),
                    }
                    .into());
                }

                let mut typed_args = Vec::new();
                for (arg, expected_ty) in args.iter().zip(param_types.iter()) {
                    let typed = self.check_expr(arg)?;
                    let arg_ty = typed.ty.clone().unwrap_or(Type::Unknown);

                    if !self.types_compatible(expected_ty, &arg_ty) {
                        return Err(CompileError::TypeError {
                            message: format!(
                                "Argument type mismatch: expected {}, found {}",
                                expected_ty, arg_ty
                            ),
                            span: arg.span.into(),
                        }
                        .into());
                    }

                    typed_args.push(typed);
                }

                (
                    ExprKind::Call {
                        name: name.clone(),
                        args: typed_args,
                    },
                    return_ty,
                )
            }

            ExprKind::Ref(inner) => {
                let inner_typed = self.check_expr(inner)?;
                let inner_ty = inner_typed.ty.clone().unwrap_or(Type::Unknown);
                (
                    ExprKind::Ref(Box::new(inner_typed)),
                    Type::Ref(Box::new(inner_ty)),
                )
            }

            ExprKind::RefMut(inner) => {
                let inner_typed = self.check_expr(inner)?;
                let inner_ty = inner_typed.ty.clone().unwrap_or(Type::Unknown);
                (
                    ExprKind::RefMut(Box::new(inner_typed)),
                    Type::RefMut(Box::new(inner_ty)),
                )
            }

            ExprKind::Assign { target, value } => {
                let value_typed = self.check_expr(value)?;
                let value_ty = value_typed.ty.clone().unwrap_or(Type::Unknown);

                if let Some(var_ty) = self.lookup_var(target).cloned() {
                    if !self.types_compatible(&var_ty, &value_ty) {
                        return Err(CompileError::TypeError {
                            message: format!(
                                "Cannot assign {} to variable of type {}",
                                value_ty, var_ty
                            ),
                            span: expr.span.into(),
                        }
                        .into());
                    }
                }

                (
                    ExprKind::Assign {
                        target: target.clone(),
                        value: Box::new(value_typed),
                    },
                    value_ty,
                )
            }

            ExprKind::PrefixOp { op, target } => {
                let ty = self.lookup_var(target).cloned().unwrap_or(Type::Int);
                (
                    ExprKind::PrefixOp {
                        op: *op,
                        target: target.clone(),
                    },
                    ty,
                )
            }

            ExprKind::PostfixOp { op, target } => {
                let ty = self.lookup_var(target).cloned().unwrap_or(Type::Int);
                (
                    ExprKind::PostfixOp {
                        op: *op,
                        target: target.clone(),
                    },
                    ty,
                )
            }
        };

        Ok(Expr {
            kind,
            span: expr.span,
            ty: Some(ty),
        })
    }

    fn binary_result_type(
        &self,
        op: BinaryOp,
        left: &Type,
        right: &Type,
        span: Span,
    ) -> Result<Type> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                match (left, right) {
                    (Type::Int, Type::Int) => Ok(Type::Int),
                    (Type::Float, Type::Float) => Ok(Type::Float),
                    (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
                    _ => Err(CompileError::TypeError {
                        message: format!("Cannot apply {} to {} and {}", op, left, right),
                        span: span.into(),
                    }
                    .into()),
                }
            }

            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => Ok(Type::Bool),

            BinaryOp::And | BinaryOp::Or => {
                if *left == Type::Bool && *right == Type::Bool {
                    Ok(Type::Bool)
                } else {
                    Err(CompileError::TypeError {
                        message: format!("Cannot apply {} to {} and {}", op, left, right),
                        span: span.into(),
                    }
                    .into())
                }
            }

            BinaryOp::Concat => {
                if *left == Type::String && *right == Type::String {
                    Ok(Type::String)
                } else {
                    Err(CompileError::TypeError {
                        message: format!("Cannot concatenate {} and {}", left, right),
                        span: span.into(),
                    }
                    .into())
                }
            }
        }
    }

    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if *expected == *actual {
            return true;
        }

        // Allow int -> float coercion
        if *expected == Type::Float && *actual == Type::Int {
            return true;
        }

        // Unknown is compatible with anything (type inference pending)
        if *expected == Type::Unknown || *actual == Type::Unknown {
            return true;
        }

        false
    }
}
