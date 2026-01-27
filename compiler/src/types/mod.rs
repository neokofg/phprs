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

use crate::ast::{Function, Program, QualifiedName, Type, Visibility};
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
    // Register traits first so classes can use them
    registry.register_traits(&program.traits);
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
    /// Current namespace
    current_namespace: Option<QualifiedName>,
    /// Import aliases: simple name -> qualified name
    import_aliases: HashMap<String, QualifiedName>,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            variables: vec![HashMap::new()],
            class_registry: ClassRegistry::new(),
            current_class: None,
            current_namespace: None,
            import_aliases: HashMap::new(),
        }
    }

    /// Set up the type checker for a compilation unit
    fn setup_for_unit(&mut self, unit: &crate::ast::CompilationUnit) {
        // Set current namespace
        self.current_namespace = unit.namespace.as_ref().map(|ns| ns.name.clone());

        // Register imports
        self.import_aliases.clear();
        for use_decl in &unit.uses {
            for item in &use_decl.items {
                let alias = item.imported_name().to_string();
                self.import_aliases.insert(alias, item.path.clone());
            }
        }
    }

    /// Resolve a type name to its qualified form
    pub(crate) fn resolve_type_name(&self, name: &str) -> String {
        // Check import aliases first
        if let Some(qn) = self.import_aliases.get(name) {
            return qn.full_path();
        }

        // If in a namespace, qualify the name
        if let Some(ns) = &self.current_namespace {
            let mut segments = ns.segments.clone();
            segments.push(name.to_string());
            return segments.join("\\");
        }

        // Return as-is (global namespace)
        name.to_string()
    }

    /// Resolve a qualified name
    #[allow(dead_code)]
    pub(crate) fn resolve_qualified_name(&self, qn: &QualifiedName) -> String {
        // If absolute or multi-segment, return as-is
        if qn.is_absolute || qn.segments.len() > 1 {
            return qn.full_path();
        }

        // Single segment - try to resolve
        if let Some(name) = qn.segments.first() {
            return self.resolve_type_name(name);
        }

        qn.full_path()
    }

    /// Resolve a Type, converting class names to fully qualified names
    pub(crate) fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Class(name) => {
                let resolved = self.resolve_type_name(name);
                // Only use resolved name if the class exists
                if self.class_registry.class_exists(&resolved) {
                    Type::Class(resolved)
                } else if self.class_registry.class_exists(name) {
                    ty.clone()
                } else {
                    Type::Class(resolved)
                }
            }
            Type::Nullable(inner) => Type::Nullable(Box::new(self.resolve_type(inner))),
            Type::Array(inner) => Type::Array(Box::new(self.resolve_type(inner))),
            Type::Ref(inner) => Type::Ref(Box::new(self.resolve_type(inner))),
            Type::RefMut(inner) => Type::RefMut(Box::new(self.resolve_type(inner))),
            _ => ty.clone(),
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
            if self
                .class_registry
                .is_subclass(actual_class, expected_class)
            {
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
        // First pass: register all traits (needed before classes that use them)
        self.class_registry.register_traits(&program.traits);

        // Second pass: register all classes (with qualified names if available)
        self.class_registry.register_classes(&program.classes);

        // Second pass: collect function signatures
        for func in &program.functions {
            let param_types: Vec<Type> = func.params.iter().map(|p| p.ty.clone()).collect();
            self.functions
                .insert(func.name.clone(), (param_types, func.return_type.clone()));
        }

        // Also register method signatures as functions (for calls)
        for class in &program.classes {
            // Use qualified name for mangling if available
            let class_key = class
                .qualified_name
                .as_ref()
                .map_or_else(|| class.name.clone(), QualifiedName::mangle);

            for method in &class.methods {
                let mangled_name = format!("{}_{}", class_key, method.name);
                let param_types: Vec<Type> = method.params.iter().map(|p| p.ty.clone()).collect();
                self.functions
                    .insert(mangled_name, (param_types, method.return_type.clone()));
            }
        }

        // Third pass: type check classes and update with typed expressions
        // Set up context for each unit if we have units
        let mut typed_classes = Vec::new();
        for class in &program.classes {
            // Find which unit this class belongs to and set up context
            if let Some(unit) = program
                .units
                .iter()
                .find(|u| u.classes.iter().any(|c| c.name == class.name))
            {
                self.setup_for_unit(unit);
            }
            typed_classes.push(self.check_class(class)?);
        }

        // Fourth pass: type check function bodies
        let mut typed_functions = Vec::new();
        for func in &program.functions {
            // Find which unit this function belongs to and set up context
            if let Some(unit) = program
                .units
                .iter()
                .find(|u| u.functions.iter().any(|f| f.name == func.name))
            {
                self.setup_for_unit(unit);
            }
            typed_functions.push(self.check_function(func)?);
        }

        Ok(Program {
            units: program.units.clone(),
            functions: typed_functions,
            classes: typed_classes,
            traits: program.traits.clone(),
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

        // Set current class context (use qualified name if available)
        self.current_class = Some(
            class
                .qualified_name
                .as_ref()
                .map_or_else(|| class.name.clone(), QualifiedName::full_path),
        );

        // Type check methods
        let mut typed_methods = Vec::new();
        for method in &class.methods {
            typed_methods.push(self.check_method(class, method)?);
        }

        self.current_class = None;

        Ok(crate::ast::ClassDef {
            name: class.name.clone(),
            qualified_name: class.qualified_name.clone(),
            parent: class.parent.clone(),
            parent_qualified: class.parent_qualified.clone(),
            interfaces: class.interfaces.clone(),
            interfaces_qualified: class.interfaces_qualified.clone(),
            properties: class.properties.clone(),
            methods: typed_methods,
            trait_uses: class.trait_uses.clone(),
            is_abstract: class.is_abstract,
            is_final: class.is_final,
            attributes: class.attributes.clone(),
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

        // Resolve return type and parameter types
        let resolved_return_type = self.resolve_type(&method.return_type);
        let resolved_params: Vec<crate::ast::Param> = method
            .params
            .iter()
            .map(|p| crate::ast::Param {
                name: p.name.clone(),
                ty: self.resolve_type(&p.ty),
                is_ref: p.is_ref,
                span: p.span,
            })
            .collect();

        // Abstract methods have no body to check
        if method.is_abstract {
            if method.body.is_some() {
                return Err(CompileError::TypeError {
                    message: "Abstract method cannot have a body".to_string(),
                    span: method.span.into(),
                }
                .into());
            }
            return Ok(crate::ast::Method {
                name: method.name.clone(),
                params: resolved_params,
                return_type: resolved_return_type,
                visibility: method.visibility,
                is_static: method.is_static,
                is_abstract: method.is_abstract,
                is_final: method.is_final,
                body: None,
                attributes: method.attributes.clone(),
                span: method.span,
            });
        }

        // Check method body
        let typed_body = if let Some(body) = &method.body {
            self.push_scope();

            // Add $this to scope (not for static methods)
            if !method.is_static {
                let class_type_name = class
                    .qualified_name
                    .as_ref()
                    .map_or_else(|| class.name.clone(), QualifiedName::full_path);
                self.define_var("this", Type::Class(class_type_name));
            }

            // Add parameters to scope with resolved types
            for param in &resolved_params {
                self.define_var(&param.name, param.ty.clone());
            }

            // Type check body with resolved return type
            let mut typed_stmts = Vec::new();
            for stmt in body {
                typed_stmts.push(self.check_stmt(stmt, &resolved_return_type)?);
            }

            self.pop_scope();
            Some(typed_stmts)
        } else {
            None
        };

        Ok(crate::ast::Method {
            name: method.name.clone(),
            params: resolved_params,
            return_type: resolved_return_type,
            visibility: method.visibility,
            is_static: method.is_static,
            is_abstract: method.is_abstract,
            is_final: method.is_final,
            body: typed_body,
            attributes: method.attributes.clone(),
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
            attributes: func.attributes.clone(),
            span: func.span,
        })
    }
}
