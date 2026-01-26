//! Expression type checking

#![allow(clippy::too_many_lines, clippy::option_if_let_else)]

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
            ExprKind::This => {
                // Resolve $this type to current class
                let this_type = if let Some(class_name) = &self.current_class {
                    Type::Class(class_name.clone())
                } else {
                    Type::SelfType
                };
                (ExprKind::This, this_type)
            }
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
            } => {
                // Look up property type from class registry
                let prop_ty = self
                    .class_registry
                    .get_property(class_name, property)
                    .map_or(Type::Unknown, |p| p.ty.clone());
                (
                    ExprKind::StaticPropertyAccess {
                        class_name: class_name.clone(),
                        property: property.clone(),
                    },
                    prop_ty,
                )
            }
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
            ExprKind::StaticPropertyAssign {
                class_name,
                property,
                value,
            } => {
                let typed_value = self.check_expr(value)?;
                let value_ty = typed_value.ty.clone().unwrap_or(Type::Unknown);
                (
                    ExprKind::StaticPropertyAssign {
                        class_name: class_name.clone(),
                        property: property.clone(),
                        value: Box::new(typed_value),
                    },
                    value_ty,
                )
            }
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
        // Resolve class name through imports/namespace
        let resolved_class = self.resolve_type_name(class_name);

        // Check class exists (try resolved name first, then original)
        let actual_class = if self.class_registry.class_exists(&resolved_class) {
            resolved_class
        } else if self.class_registry.class_exists(class_name) {
            class_name.to_string()
        } else {
            return Err(CompileError::TypeError {
                message: format!("Class '{class_name}' not found"),
                span: Span::default().into(),
            }
            .into());
        };

        // Check if class is abstract
        if let Some(class_info) = self.class_registry.get_class(&actual_class) {
            if class_info.is_abstract {
                return Err(CompileError::TypeError {
                    message: format!("Cannot instantiate abstract class '{actual_class}'"),
                    span: Span::default().into(),
                }
                .into());
            }
        }

        // Get constructor parameter types
        let constructor_params: Option<Vec<Type>> = self
            .class_registry
            .get_constructor(&actual_class)
            .map(|c| c.params.iter().map(|(_, ty)| ty.clone()).collect());

        // Type check constructor arguments
        let mut typed_args = Vec::new();
        if let Some(param_types) = constructor_params {
            if args.len() != param_types.len() {
                return Err(CompileError::TypeError {
                    message: format!(
                        "Constructor of '{}' expects {} arguments, got {}",
                        actual_class,
                        param_types.len(),
                        args.len()
                    ),
                    span: Span::default().into(),
                }
                .into());
            }

            for (arg, expected_ty) in args.iter().zip(param_types.iter()) {
                let typed = self.check_expr(arg)?;
                let arg_ty = typed.ty.clone().unwrap_or(Type::Unknown);

                if !self.types_compatible(expected_ty, &arg_ty) {
                    return Err(CompileError::TypeError {
                        message: format!(
                            "Constructor argument type mismatch: expected {expected_ty}, found {arg_ty}"
                        ),
                        span: arg.span.into(),
                    }
                    .into());
                }

                typed_args.push(typed);
            }
        } else {
            // No constructor - no args expected
            if !args.is_empty() {
                return Err(CompileError::TypeError {
                    message: format!(
                        "Class '{}' has no constructor but {} arguments were provided",
                        actual_class,
                        args.len()
                    ),
                    span: Span::default().into(),
                }
                .into());
            }
        }

        Ok((
            ExprKind::New {
                class_name: actual_class.clone(),
                args: typed_args,
            },
            Type::Class(actual_class),
        ))
    }

    fn check_property_access(
        &mut self,
        object: &Expr,
        property: &str,
    ) -> Result<(ExprKind, Type)> {
        let object_typed = self.check_expr(object)?;
        let object_ty = object_typed.ty.clone().unwrap_or(Type::Unknown);

        let property_ty = if let Type::Class(class_name) = &object_ty {
            if let Some(prop_info) = self.class_registry.get_property(class_name, property) {
                // Check visibility
                self.check_visibility(class_name, prop_info.visibility, object.span)?;

                // Check not accessing static property via instance
                if prop_info.is_static {
                    return Err(CompileError::TypeError {
                        message: format!(
                            "Cannot access static property '{property}' via instance, use {class_name}::${property}"
                        ),
                        span: object.span.into(),
                    }
                    .into());
                }

                prop_info.ty.clone()
            } else {
                return Err(CompileError::TypeError {
                    message: format!("Property '{property}' not found in class '{class_name}'"),
                    span: object.span.into(),
                }
                .into());
            }
        } else {
            Type::Unknown
        };

        Ok((
            ExprKind::PropertyAccess {
                object: Box::new(object_typed),
                property: property.to_string(),
            },
            property_ty,
        ))
    }

    fn check_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<(ExprKind, Type)> {
        let object_typed = self.check_expr(object)?;
        let object_ty = object_typed.ty.clone().unwrap_or(Type::Unknown);

        let (typed_args, return_ty) = if let Type::Class(class_name) = &object_ty {
            // Extract method info before mutable borrow
            let method_data = self.class_registry.get_method(class_name, method).map(|m| {
                (
                    m.visibility,
                    m.is_static,
                    m.params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>(),
                    m.return_type.clone(),
                )
            });

            if let Some((visibility, is_static, param_types, return_type)) = method_data {
                // Check visibility
                self.check_visibility(class_name, visibility, object.span)?;

                // Check not calling static method via instance
                if is_static {
                    return Err(CompileError::TypeError {
                        message: format!(
                            "Cannot call static method '{method}' via instance, use {class_name}::{method}()"
                        ),
                        span: object.span.into(),
                    }
                    .into());
                }

                // Check argument count
                if args.len() != param_types.len() {
                    return Err(CompileError::TypeError {
                        message: format!(
                            "Method '{}' expects {} arguments, got {}",
                            method,
                            param_types.len(),
                            args.len()
                        ),
                        span: object.span.into(),
                    }
                    .into());
                }

                // Type check arguments
                let mut typed = Vec::new();
                for (arg, expected_ty) in args.iter().zip(param_types.iter()) {
                    let arg_typed = self.check_expr(arg)?;
                    let arg_ty = arg_typed.ty.clone().unwrap_or(Type::Unknown);

                    // Resolve expected type for comparison
                    let resolved_expected = self.resolve_type(&expected_ty);
                    if !self.types_compatible(&resolved_expected, &arg_ty) {
                        return Err(CompileError::TypeError {
                            message: format!(
                                "Method argument type mismatch: expected {resolved_expected}, found {arg_ty}"
                            ),
                            span: arg.span.into(),
                        }
                        .into());
                    }

                    typed.push(arg_typed);
                }

                // Resolve return type
                let resolved_return = self.resolve_type(&return_type);
                (typed, resolved_return)
            } else {
                return Err(CompileError::TypeError {
                    message: format!("Method '{method}' not found in class '{class_name}'"),
                    span: object.span.into(),
                }
                .into());
            }
        } else {
            // Unknown type - just type check args
            let mut typed = Vec::new();
            for arg in args {
                typed.push(self.check_expr(arg)?);
            }
            (typed, Type::Unknown)
        };

        Ok((
            ExprKind::MethodCall {
                object: Box::new(object_typed),
                method: method.to_string(),
                args: typed_args,
            },
            return_ty,
        ))
    }

    fn check_static_method_call(
        &mut self,
        class_name: &str,
        method: &str,
        args: &[Expr],
    ) -> Result<(ExprKind, Type)> {
        // Handle parent:: calls
        let resolved_class = if class_name == "parent" {
            if let Some(current) = &self.current_class {
                if let Some(class_info) = self.class_registry.get_class(current) {
                    class_info.parent.clone().ok_or_else(|| CompileError::TypeError {
                        message: "Cannot use parent:: - class has no parent".to_string(),
                        span: Span::default().into(),
                    })?
                } else {
                    return Err(CompileError::TypeError {
                        message: "Cannot use parent:: outside of class".to_string(),
                        span: Span::default().into(),
                    }
                    .into());
                }
            } else {
                return Err(CompileError::TypeError {
                    message: "Cannot use parent:: outside of class".to_string(),
                    span: Span::default().into(),
                }
                .into());
            }
        } else {
            class_name.to_string()
        };

        // Check class exists
        if !self.class_registry.class_exists(&resolved_class) {
            return Err(CompileError::TypeError {
                message: format!("Class '{resolved_class}' not found"),
                span: Span::default().into(),
            }
            .into());
        }

        // Extract method info before mutable borrow
        let method_data = self
            .class_registry
            .get_method(&resolved_class, method)
            .map(|m| {
                (
                    m.visibility,
                    m.params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>(),
                    m.return_type.clone(),
                )
            });

        let (typed_args, return_ty) = if let Some((visibility, param_types, return_type)) =
            method_data
        {
            // Check visibility
            self.check_visibility(&resolved_class, visibility, Span::default())?;

            // Check argument count
            if args.len() != param_types.len() {
                return Err(CompileError::TypeError {
                    message: format!(
                        "Method '{}::{}' expects {} arguments, got {}",
                        resolved_class,
                        method,
                        param_types.len(),
                        args.len()
                    ),
                    span: Span::default().into(),
                }
                .into());
            }

            let mut typed = Vec::new();
            for (arg, expected_ty) in args.iter().zip(param_types.iter()) {
                let arg_typed = self.check_expr(arg)?;
                let arg_ty = arg_typed.ty.clone().unwrap_or(Type::Unknown);

                if !self.types_compatible(expected_ty, &arg_ty) {
                    return Err(CompileError::TypeError {
                        message: format!(
                            "Static method argument type mismatch: expected {expected_ty}, found {arg_ty}"
                        ),
                        span: arg.span.into(),
                    }
                    .into());
                }

                typed.push(arg_typed);
            }

            (typed, return_type)
        } else {
            return Err(CompileError::TypeError {
                message: format!(
                    "Static method '{method}' not found in class '{resolved_class}'"
                ),
                span: Span::default().into(),
            }
            .into());
        };

        Ok((
            ExprKind::StaticMethodCall {
                class_name: resolved_class,
                method: method.to_string(),
                args: typed_args,
            },
            return_ty,
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
        let object_ty = object_typed.ty.clone().unwrap_or(Type::Unknown);
        let value_ty = value_typed.ty.clone().unwrap_or(Type::Unknown);

        // Check property exists and types match
        if let Type::Class(class_name) = &object_ty {
            if let Some(prop_info) = self.class_registry.get_property(class_name, property) {
                // Check visibility
                self.check_visibility(class_name, prop_info.visibility, object.span)?;

                // Check type compatibility
                if !self.types_compatible(&prop_info.ty, &value_ty) {
                    return Err(CompileError::TypeError {
                        message: format!(
                            "Cannot assign {} to property '{}' of type {}",
                            value_ty, property, prop_info.ty
                        ),
                        span: value.span.into(),
                    }
                    .into());
                }
            } else {
                return Err(CompileError::TypeError {
                    message: format!("Property '{property}' not found in class '{class_name}'"),
                    span: object.span.into(),
                }
                .into());
            }
        }

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
