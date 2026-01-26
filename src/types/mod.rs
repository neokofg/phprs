//! Type checking module

#![allow(
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::unused_self,
    clippy::branches_sharing_code,
    clippy::doc_markdown,
    clippy::redundant_pub_crate
)]

mod expr;
mod stmt;

use std::collections::HashMap;

use crate::ast::{Function, Program, Type};
use miette::Result;

/// Type check a program.
pub fn check(program: &Program) -> Result<Program> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}

pub(crate) struct TypeChecker {
    /// Function signatures: name -> (params, return_type)
    pub(crate) functions: HashMap<String, (Vec<Type>, Type)>,
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

        false
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
            classes: program.classes.clone(),
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
