//! Expression type checking

use crate::ast::{ArrayElement, BinaryOp, Expr, ExprKind, Span, Type, UnaryOp};
use crate::errors::CompileError;
use miette::Result;

use super::TypeChecker;

impl TypeChecker {
    pub(super) fn check_expr(&mut self, expr: &Expr) -> Result<Expr> {
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

            ExprKind::Binary { left, op, right } => self.check_binary(left, *op, right, expr.span)?,

            ExprKind::Unary { op, operand } => self.check_unary(*op, operand, expr.span)?,

            ExprKind::Call { name, args } => self.check_call(name, args, expr.span)?,

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

            ExprKind::Assign { target, value } => self.check_assign_expr(target, value, expr.span)?,

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

            // OOP Expressions
            ExprKind::New { class_name, args } => self.check_new(class_name, args)?,
            ExprKind::This => (ExprKind::This, Type::SelfType),
            ExprKind::PropertyAccess { object, property } => {
                self.check_property_access(object, property)?
            }
            ExprKind::MethodCall {
                object,
                method,
                args,
            } => self.check_method_call(object, method, args)?,
            ExprKind::StaticPropertyAccess {
                class_name,
                property,
            } => (
                ExprKind::StaticPropertyAccess {
                    class_name: class_name.clone(),
                    property: property.clone(),
                },
                Type::Unknown,
            ),
            ExprKind::StaticMethodCall {
                class_name,
                method,
                args,
            } => self.check_static_method_call(class_name, method, args)?,
            ExprKind::PropertyAssign {
                object,
                property,
                value,
            } => self.check_property_assign(object, property, value)?,
            ExprKind::ArrayLit(elements) => self.check_array_lit(elements)?,
            ExprKind::ArrayAccess { array, index } => self.check_array_access(array, index)?,
        };

        Ok(Expr {
            kind,
            span: expr.span,
            ty: Some(ty),
        })
    }

    fn check_binary(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: Span,
    ) -> Result<(ExprKind, Type)> {
        let left_typed = self.check_expr(left)?;
        let right_typed = self.check_expr(right)?;

        let left_ty = left_typed.ty.clone().unwrap_or(Type::Unknown);
        let right_ty = right_typed.ty.clone().unwrap_or(Type::Unknown);

        let result_ty = self.binary_result_type(op, &left_ty, &right_ty, span)?;

        Ok((
            ExprKind::Binary {
                left: Box::new(left_typed),
                op,
                right: Box::new(right_typed),
            },
            result_ty,
        ))
    }

    fn check_unary(&mut self, op: UnaryOp, operand: &Expr, span: Span) -> Result<(ExprKind, Type)> {
        let operand_typed = self.check_expr(operand)?;
        let operand_ty = operand_typed.ty.clone().unwrap_or(Type::Unknown);

        let result_ty = match op {
            UnaryOp::Neg => {
                if matches!(operand_ty, Type::Int | Type::Float) {
                    operand_ty
                } else {
                    return Err(CompileError::TypeError {
                        message: format!("Cannot negate type {operand_ty}"),
                        span: span.into(),
                    }
                    .into());
                }
            }
            UnaryOp::Not => Type::Bool,
            UnaryOp::Inc | UnaryOp::Dec => operand_ty,
        };

        Ok((
            ExprKind::Unary {
                op,
                operand: Box::new(operand_typed),
            },
            result_ty,
        ))
    }

    fn check_call(&mut self, name: &str, args: &[Expr], span: Span) -> Result<(ExprKind, Type)> {
        let (param_types, return_ty) =
            self.functions
                .get(name)
                .cloned()
                .ok_or_else(|| CompileError::TypeError {
                    message: format!("Unknown function: {name}"),
                    span: span.into(),
                })?;

        if args.len() != param_types.len() {
            return Err(CompileError::TypeError {
                message: format!(
                    "Function {} expects {} arguments, got {}",
                    name,
                    param_types.len(),
                    args.len()
                ),
                span: span.into(),
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
                        "Argument type mismatch: expected {expected_ty}, found {arg_ty}"
                    ),
                    span: arg.span.into(),
                }
                .into());
            }

            typed_args.push(typed);
        }

        Ok((
            ExprKind::Call {
                name: name.to_string(),
                args: typed_args,
            },
            return_ty,
        ))
    }

    fn check_assign_expr(
        &mut self,
        target: &str,
        value: &Expr,
        span: Span,
    ) -> Result<(ExprKind, Type)> {
        let value_typed = self.check_expr(value)?;
        let value_ty = value_typed.ty.clone().unwrap_or(Type::Unknown);

        if let Some(var_ty) = self.lookup_var(target).cloned() {
            if !self.types_compatible(&var_ty, &value_ty) {
                return Err(CompileError::TypeError {
                    message: format!("Cannot assign {value_ty} to variable of type {var_ty}"),
                    span: span.into(),
                }
                .into());
            }
        }

        Ok((
            ExprKind::Assign {
                target: target.to_string(),
                value: Box::new(value_typed),
            },
            value_ty,
        ))
    }

    fn check_new(&mut self, class_name: &str, args: &[Expr]) -> Result<(ExprKind, Type)> {
        let mut typed_args = Vec::new();
        for arg in args {
            typed_args.push(self.check_expr(arg)?);
        }
        Ok((
            ExprKind::New {
                class_name: class_name.to_string(),
                args: typed_args,
            },
            Type::Class(class_name.to_string()),
        ))
    }

    fn check_property_access(
        &mut self,
        object: &Expr,
        property: &str,
    ) -> Result<(ExprKind, Type)> {
        let object_typed = self.check_expr(object)?;
        Ok((
            ExprKind::PropertyAccess {
                object: Box::new(object_typed),
                property: property.to_string(),
            },
            Type::Unknown,
        ))
    }

    fn check_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<(ExprKind, Type)> {
        let object_typed = self.check_expr(object)?;
        let mut typed_args = Vec::new();
        for arg in args {
            typed_args.push(self.check_expr(arg)?);
        }
        Ok((
            ExprKind::MethodCall {
                object: Box::new(object_typed),
                method: method.to_string(),
                args: typed_args,
            },
            Type::Unknown,
        ))
    }

    fn check_static_method_call(
        &mut self,
        class_name: &str,
        method: &str,
        args: &[Expr],
    ) -> Result<(ExprKind, Type)> {
        let mut typed_args = Vec::new();
        for arg in args {
            typed_args.push(self.check_expr(arg)?);
        }
        Ok((
            ExprKind::StaticMethodCall {
                class_name: class_name.to_string(),
                method: method.to_string(),
                args: typed_args,
            },
            Type::Unknown,
        ))
    }

    fn check_property_assign(
        &mut self,
        object: &Expr,
        property: &str,
        value: &Expr,
    ) -> Result<(ExprKind, Type)> {
        let object_typed = self.check_expr(object)?;
        let value_typed = self.check_expr(value)?;
        let value_ty = value_typed.ty.clone().unwrap_or(Type::Unknown);
        Ok((
            ExprKind::PropertyAssign {
                object: Box::new(object_typed),
                property: property.to_string(),
                value: Box::new(value_typed),
            },
            value_ty,
        ))
    }

    fn check_array_lit(&mut self, elements: &[ArrayElement]) -> Result<(ExprKind, Type)> {
        let mut typed_elements = Vec::new();
        let mut elem_ty = Type::Unknown;

        for elem in elements {
            let typed_key = if let Some(key) = &elem.key {
                Some(self.check_expr(key)?)
            } else {
                None
            };
            let typed_value = self.check_expr(&elem.value)?;

            if elem_ty == Type::Unknown {
                elem_ty = typed_value.ty.clone().unwrap_or(Type::Unknown);
            }

            typed_elements.push(ArrayElement {
                key: typed_key,
                value: typed_value,
            });
        }

        Ok((
            ExprKind::ArrayLit(typed_elements),
            Type::Array(Box::new(elem_ty)),
        ))
    }

    fn check_array_access(&mut self, array: &Expr, index: &Expr) -> Result<(ExprKind, Type)> {
        let array_typed = self.check_expr(array)?;
        let index_typed = self.check_expr(index)?;

        let elem_ty = if let Some(Type::Array(inner)) = &array_typed.ty {
            (**inner).clone()
        } else {
            Type::Unknown
        };

        Ok((
            ExprKind::ArrayAccess {
                array: Box::new(array_typed),
                index: Box::new(index_typed),
            },
            elem_ty,
        ))
    }

    pub(super) fn binary_result_type(
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
                        message: format!("Cannot apply {op} to {left} and {right}"),
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
                        message: format!("Cannot apply {op} to {left} and {right}"),
                        span: span.into(),
                    }
                    .into())
                }
            }

            BinaryOp::Concat => Ok(Type::String),
        }
    }
}
