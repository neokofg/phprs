use super::{QualifiedName, Span, Stmt, TraitUse, Type};

/// Visibility modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Public,
    Private,
    Protected,
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Private => write!(f, "private"),
            Self::Protected => write!(f, "protected"),
        }
    }
}

/// Class property
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Property {
    pub name: String,
    pub ty: Type,
    pub visibility: Visibility,
    pub is_static: bool,
    pub default: Option<super::Expr>,
    pub span: Span,
}

/// Class method
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Method {
    pub name: String,
    pub params: Vec<super::stmt::Param>,
    pub return_type: Type,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub body: Option<Vec<Stmt>>,
    pub span: Span,
}

impl Method {
    /// Check if this is a constructor
    #[must_use]
    #[allow(dead_code)]
    pub fn is_constructor(&self) -> bool {
        self.name == "__construct"
    }
}

/// Class definition
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClassDef {
    /// Simple class name
    pub name: String,
    /// Fully qualified name (with namespace)
    pub qualified_name: Option<QualifiedName>,
    /// Parent class (can be qualified)
    pub parent: Option<String>,
    /// Parent class as qualified name (for resolution)
    pub parent_qualified: Option<QualifiedName>,
    /// Implemented interfaces
    pub interfaces: Vec<String>,
    /// Interfaces as qualified names (for resolution)
    pub interfaces_qualified: Vec<QualifiedName>,
    /// Class properties
    pub properties: Vec<Property>,
    /// Class methods
    pub methods: Vec<Method>,
    /// Trait uses (class-level use statements)
    pub trait_uses: Vec<TraitUse>,
    pub is_abstract: bool,
    pub is_final: bool,
    pub span: Span,
}

#[allow(dead_code)]
impl ClassDef {
    /// Get constructor method if exists
    #[must_use]
    pub fn constructor(&self) -> Option<&Method> {
        self.methods.iter().find(|m| m.is_constructor())
    }

    /// Get method by name
    #[must_use]
    pub fn get_method(&self, name: &str) -> Option<&Method> {
        self.methods.iter().find(|m| m.name == name)
    }

    /// Get property by name
    #[must_use]
    pub fn get_property(&self, name: &str) -> Option<&Property> {
        self.properties.iter().find(|p| p.name == name)
    }
}

/// Interface definition
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InterfaceDef {
    pub name: String,
    pub extends: Vec<String>,
    pub methods: Vec<Method>,
    pub span: Span,
}

/// Trait definition
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TraitDef {
    /// Simple trait name
    pub name: String,
    /// Fully qualified name (with namespace)
    pub qualified_name: Option<QualifiedName>,
    /// Trait properties
    pub properties: Vec<Property>,
    /// Trait methods
    pub methods: Vec<Method>,
    pub span: Span,
}
