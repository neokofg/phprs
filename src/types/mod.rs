//! Type checking module

#![allow(
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::unused_self,
    clippy::branches_sharing_code,
    clippy::doc_markdown,
    clippy::redundant_pub_crate
)]

mod class_registry;
mod expr;
mod stmt;

pub use class_registry::ClassRegistry;

use std::collections::HashMap;

use crate::ast::{Function, Program, Type, Visibility};
use crate::errors::CompileError;
use miette::Result;

/// Type check a program.
pub fn check(program: &Program) -> Result<Program> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}

/// Get the class registry from a program (for codegen)
#[must_use] 
pub fn build_class_registry(program: &Program) -> ClassRegistry {
    let mut registry = ClassRegistry::new();
    registry.register_classes(&program.classes);
    registry
}

pub(crate) struct TypeChecker {
    /// Function signatures: name -> (params, return_type)
    pub(crate) functions: HashMap<String, (Vec<Type>, Type)>,
    /// Variable types in current scope
    variables: Vec<HashMap<String, Type>>,
    /// Class registry for OOP support
    pub(crate) class_registry: ClassRegistry,
    /// Current class context (for $this, visibility checks)
    pub(crate) current_class: Option<String>,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            variables: vec![HashMap::new()],
            class_registry: ClassRegistry::new(),
            current_class: None,
        }
    }

    pub(crate) fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        self.variables.pop();
    }

    pub(crate) fn define_var(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    pub(crate) fn lookup_var(&self, name: &str) -> Option<&Type> {
        for scope in self.variables.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    pub(crate) fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
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

        // Class subtyping: child class is compatible with parent type
        if let (Type::Class(expected_class), Type::Class(actual_class)) = (expected, actual) {
            if self.class_registry.is_subclass(actual_class, expected_class) {
                return true;
            }
        }

        // Nullable types
        if let Type::Nullable(inner) = expected {
            if self.types_compatible(inner, actual) {
                return true;
            }
        }

        false
    }

    /// Check if a member is accessible from current context
    pub(crate) fn check_visibility(
        &self,
        target_class: &str,
        visibility: Visibility,
        span: crate::ast::Span,
    ) -> Result<()> {
        let accessible = self.class_registry.is_accessible(
            target_class,
            visibility,
            self.current_class.as_deref(),
        );

        if !accessible {
            return Err(CompileError::TypeError {
                message: format!(
                    "Cannot access {} member from {}",
                    visibility,
                    self.current_class.as_deref().unwrap_or("outside class")
                ),
                span: span.into(),
            }
            .into());
        }

        Ok(())
    }

    fn check_program(&mut self, program: &Program) -> Result<Program> {
        // First pass: register all classes
        self.class_registry.register_classes(&program.classes);

        // Second pass: collect function signatures
        for func in &program.functions {
            let param_types: Vec<Type> = func.params.iter().map(|p| p.ty.clone()).collect();
            self.functions
                .insert(func.name.clone(), (param_types, func.return_type.clone()));
        }

        // Also register method signatures as functions (for calls)
        for class in &program.classes {
            for method in &class.methods {
                let mangled_name = format!("{}_{}", class.name, method.name);
                let param_types: Vec<Type> = method.params.iter().map(|p| p.ty.clone()).collect();
                self.functions
                    .insert(mangled_name, (param_types, method.return_type.clone()));
            }
        }

        // Third pass: type check classes and update with typed expressions
        let mut typed_classes = Vec::new();
        for class in &program.classes {
            typed_classes.push(self.check_class(class)?);
        }

        // Fourth pass: type check function bodies
        let mut typed_functions = Vec::new();
        for func in &program.functions {
            typed_functions.push(self.check_function(func)?);
        }

        Ok(Program {
            functions: typed_functions,
            classes: typed_classes,
        })
    }

    fn check_class(&mut self, class: &crate::ast::ClassDef) -> Result<crate::ast::ClassDef> {
        // Check parent class exists
        if let Some(parent) = &class.parent {
            if !self.class_registry.class_exists(parent) {
                return Err(CompileError::TypeError {
                    message: format!("Parent class '{parent}' not found"),
                    span: class.span.into(),
                }
                .into());
            }

            // Check not extending final class
            if let Some(parent_info) = self.class_registry.get_class(parent) {
                if parent_info.is_final {
                    return Err(CompileError::TypeError {
                        message: format!("Cannot extend final class '{parent}'"),
                        span: class.span.into(),
                    }
                    .into());
                }
            }
        }

        // Set current class context
        self.current_class = Some(class.name.clone());

        // Type check methods
        let mut typed_methods = Vec::new();
        for method in &class.methods {
            typed_methods.push(self.check_method(class, method)?);
        }

        self.current_class = None;

        Ok(crate::ast::ClassDef {
            name: class.name.clone(),
            parent: class.parent.clone(),
            interfaces: class.interfaces.clone(),
            properties: class.properties.clone(),
            methods: typed_methods,
            is_abstract: class.is_abstract,
            is_final: class.is_final,
            span: class.span,
        })
    }

    fn check_method(
        &mut self,
        class: &crate::ast::ClassDef,
        method: &crate::ast::Method,
    ) -> Result<crate::ast::Method> {
        // Check override constraints
        if let Some(parent) = &class.parent {
            if let Some(parent_method) = self.class_registry.get_method(parent, &method.name) {
                // Cannot override final method
                if parent_method.is_final {
                    return Err(CompileError::TypeError {
                        message: format!(
                            "Cannot override final method '{}' from parent class",
                            method.name
                        ),
                        span: method.span.into(),
                    }
                    .into());
                }

                // Check return type compatibility (covariance)
                if !self.types_compatible(&parent_method.return_type, &method.return_type) {
                    return Err(CompileError::TypeError {
                        message: format!(
                            "Override return type '{}' is not compatible with parent type '{}'",
                            method.return_type, parent_method.return_type
                        ),
                        span: method.span.into(),
                    }
                    .into());
                }
            }
        }

        // Abstract methods have no body to check
        if method.is_abstract {
            if method.body.is_some() {
                return Err(CompileError::TypeError {
                    message: "Abstract method cannot have a body".to_string(),
                    span: method.span.into(),
                }
                .into());
            }
            return Ok(method.clone());
        }

        // Check method body
        let typed_body = if let Some(body) = &method.body {
            self.push_scope();

            // Add $this to scope (not for static methods)
            if !method.is_static {
                self.define_var("this", Type::Class(class.name.clone()));
            }

            // Add parameters to scope
            for param in &method.params {
                self.define_var(&param.name, param.ty.clone());
            }

            // Type check body
            let mut typed_stmts = Vec::new();
            for stmt in body {
                typed_stmts.push(self.check_stmt(stmt, &method.return_type)?);
            }

            self.pop_scope();
            Some(typed_stmts)
        } else {
            None
        };

        Ok(crate::ast::Method {
            name: method.name.clone(),
            params: method.params.clone(),
            return_type: method.return_type.clone(),
            visibility: method.visibility,
            is_static: method.is_static,
            is_abstract: method.is_abstract,
            is_final: method.is_final,
            body: typed_body,
            span: method.span,
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
}
