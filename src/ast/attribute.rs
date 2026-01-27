//! PHP Attributes (Annotations)
//!
//! Low-level attribute system. The compiler parses and stores attributes
//! but does NOT interpret them. Frameworks decide what attributes mean.
//!
//! ```php
//! #[Route("GET", "/users")]
//! #[Cache(ttl: 60)]
//! #[Inject(UserService::class)]
//! fn list_users(): array { ... }
//! ```

use super::Span;
use super::expr::Expr;

/// A single attribute
///
/// Example: `#[Route("GET", "/users/{id}")]`
#[derive(Debug, Clone)]
pub struct Attribute {
    /// Attribute name (e.g., "Route", "Cache", "Inject")
    pub name: String,

    /// Arguments passed to the attribute
    pub args: Vec<AttributeArg>,

    /// Source location
    pub span: Span,
}

/// Attribute argument
#[derive(Debug, Clone)]
pub enum AttributeArg {
    /// Positional argument: `#[Route("GET", "/path")]`
    Positional(Expr),

    /// Named argument: `#[Cache(ttl: 60, key: "users")]`
    Named(String, Expr),
}

impl Attribute {
    /// Create a new attribute
    #[must_use]
    pub const fn new(name: String, args: Vec<AttributeArg>, span: Span) -> Self {
        Self { name, args, span }
    }

    /// Create attribute with no arguments
    #[must_use]
    pub const fn simple(name: String, span: Span) -> Self {
        Self { name, args: Vec::new(), span }
    }

    /// Get positional arguments only
    pub fn positional_args(&self) -> impl Iterator<Item = &Expr> {
        self.args.iter().filter_map(|arg| {
            match arg {
                AttributeArg::Positional(expr) => Some(expr),
                AttributeArg::Named(..) => None,
            }
        })
    }

    /// Get named argument by name
    #[must_use]
    pub fn get_named(&self, name: &str) -> Option<&Expr> {
        self.args.iter().find_map(|arg| {
            match arg {
                AttributeArg::Named(n, expr) if n == name => Some(expr),
                _ => None,
            }
        })
    }

    /// Check if attribute has a specific name (case-insensitive)
    #[must_use]
    pub fn is(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name)
    }
}

/// Collection of attributes attached to a declaration
#[derive(Debug, Clone, Default)]
pub struct Attributes {
    pub items: Vec<Attribute>,
}

impl Attributes {
    #[must_use]
    pub const fn new() -> Self {
        Self { items: Vec::new() }
    }

    #[must_use]
    pub const fn with_items(items: Vec<Attribute>) -> Self {
        Self { items }
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get attribute by name (case-insensitive)
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Attribute> {
        self.items.iter().find(|a| a.is(name))
    }

    /// Check if has attribute by name
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Get all attributes with given name
    #[must_use]
    pub fn get_all(&self, name: &str) -> Vec<&Attribute> {
        self.items.iter().filter(|a| a.is(name)).collect()
    }

    /// Iterate over all attributes
    pub fn iter(&self) -> impl Iterator<Item = &Attribute> {
        self.items.iter()
    }

    /// Add an attribute
    pub fn push(&mut self, attr: Attribute) {
        self.items.push(attr);
    }
}

impl IntoIterator for Attributes {
    type Item = Attribute;
    type IntoIter = std::vec::IntoIter<Attribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a Attributes {
    type Item = &'a Attribute;
    type IntoIter = std::slice::Iter<'a, Attribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
