//! Symbol table for namespace resolution

use std::collections::HashMap;

use crate::ast::{QualifiedName, Span, UseKind};

/// A symbol that can be imported
#[derive(Debug, Clone)]
pub enum Symbol {
    Class(QualifiedName),
    Function(QualifiedName),
    Constant(QualifiedName),
    #[allow(dead_code)]
    Trait(QualifiedName),
}

impl Symbol {
    /// Get the qualified name of this symbol
    #[must_use]
    pub const fn qualified_name(&self) -> &QualifiedName {
        match self {
            Self::Class(qn) | Self::Function(qn) | Self::Constant(qn) | Self::Trait(qn) => qn,
        }
    }
}

/// Symbol table for a compilation unit
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Current namespace
    pub namespace: Option<QualifiedName>,
    /// Imported symbols: alias -> qualified name
    imports: HashMap<String, Symbol>,
    /// Defined classes in this unit
    defined_classes: HashMap<String, QualifiedName>,
    /// Defined functions in this unit
    defined_functions: HashMap<String, QualifiedName>,
    /// Defined traits in this unit
    defined_traits: HashMap<String, QualifiedName>,
}

impl SymbolTable {
    /// Create a new symbol table
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the namespace for this unit
    pub fn set_namespace(&mut self, ns: QualifiedName) {
        self.namespace = Some(ns);
    }

    /// Add an import
    pub fn add_import(&mut self, alias: String, path: QualifiedName, kind: UseKind) {
        let symbol = match kind {
            UseKind::Class => Symbol::Class(path),
            UseKind::Function => Symbol::Function(path),
            UseKind::Const => Symbol::Constant(path),
        };
        self.imports.insert(alias, symbol);
    }

    /// Register a defined class
    pub fn define_class(&mut self, name: &str, qualified: QualifiedName) {
        self.defined_classes.insert(name.to_string(), qualified);
    }

    /// Register a defined function
    pub fn define_function(&mut self, name: &str, qualified: QualifiedName) {
        self.defined_functions.insert(name.to_string(), qualified);
    }

    /// Register a defined trait
    pub fn define_trait(&mut self, name: &str, qualified: QualifiedName) {
        self.defined_traits.insert(name.to_string(), qualified);
    }

    /// Resolve a name to its fully qualified form
    /// Returns `Some` with resolved name (always returns Some, but Option is kept for API stability)
    #[must_use]
    #[allow(clippy::unnecessary_wraps)]
    pub fn resolve(&self, name: &str, kind: UseKind) -> Option<QualifiedName> {
        // 1. Check imports
        if let Some(symbol) = self.imports.get(name) {
            if matches!(
                (kind, symbol),
                (UseKind::Class, Symbol::Class(_))
                    | (UseKind::Function, Symbol::Function(_))
                    | (UseKind::Const, Symbol::Constant(_))
            ) {
                return Some(symbol.qualified_name().clone());
            }
        }

        // 2. Check local definitions
        match kind {
            UseKind::Class => {
                if let Some(qn) = self.defined_classes.get(name) {
                    return Some(qn.clone());
                }
            }
            UseKind::Function => {
                if let Some(qn) = self.defined_functions.get(name) {
                    return Some(qn.clone());
                }
            }
            UseKind::Const => {
                // TODO: constants
            }
        }

        // 3. If in a namespace, try current namespace + name
        if let Some(ns) = &self.namespace {
            let mut segments = ns.segments.clone();
            segments.push(name.to_string());
            return Some(QualifiedName::new(segments, false, Span::default()));
        }

        // 4. Assume global namespace
        Some(QualifiedName::simple(name.to_string(), Span::default()))
    }

    /// Resolve a qualified name
    #[must_use]
    pub fn resolve_qualified(&self, qn: &QualifiedName, kind: UseKind) -> QualifiedName {
        // If absolute or multi-segment, return as-is
        if qn.is_absolute || qn.segments.len() > 1 {
            return qn.clone();
        }

        // Single segment name - try to resolve
        if let Some(name) = qn.segments.first() {
            if let Some(resolved) = self.resolve(name, kind) {
                return resolved;
            }
        }

        qn.clone()
    }

    /// Get all imports
    pub fn imports(&self) -> impl Iterator<Item = (&String, &Symbol)> {
        self.imports.iter()
    }
}
