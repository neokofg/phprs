//! Namespace and use declaration AST nodes

use super::Span;
use std::path::PathBuf;

/// Qualified name: `App\Models\User`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedName {
    /// Name segments: `["App", "Models", "User"]`
    pub segments: Vec<String>,
    /// Starts with \ (absolute path)
    pub is_absolute: bool,
    pub span: Span,
}

impl QualifiedName {
    /// Create a new qualified name
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(segments: Vec<String>, is_absolute: bool, span: Span) -> Self {
        Self {
            segments,
            is_absolute,
            span,
        }
    }

    /// Create a simple (single-segment) name
    #[must_use]
    pub fn simple(name: String, span: Span) -> Self {
        Self {
            segments: vec![name],
            is_absolute: false,
            span,
        }
    }

    /// Get the full path as a string (e.g., "App\\Models\\User")
    #[must_use]
    pub fn full_path(&self) -> String {
        let prefix = if self.is_absolute { "\\" } else { "" };
        format!("{}{}", prefix, self.segments.join("\\"))
    }

    /// Get the last segment (class/function name)
    #[must_use]
    pub fn last(&self) -> Option<&str> {
        self.segments.last().map(String::as_str)
    }

    /// Check if this is a simple (single-segment) name
    #[must_use]
    pub fn is_simple(&self) -> bool {
        self.segments.len() == 1 && !self.is_absolute
    }

    /// Convert to mangled name for codegen: `App\Models\User` → `App__Models__User`
    #[must_use]
    pub fn mangle(&self) -> String {
        self.segments.join("__")
    }
}

impl std::fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_path())
    }
}

/// namespace `App\Models`;
#[derive(Debug, Clone)]
pub struct NamespaceDecl {
    pub name: QualifiedName,
    #[allow(dead_code)]
    pub span: Span,
}

/// Type of import
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UseKind {
    #[default]
    Class, // use App\User;
    Function, // use function App\format;
    Const,    // use const App\DEBUG;
}

impl std::fmt::Display for UseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class => write!(f, "class"),
            Self::Function => write!(f, "function"),
            Self::Const => write!(f, "const"),
        }
    }
}

/// Single use item: `App\User as U`
#[derive(Debug, Clone)]
pub struct UseItem {
    /// Full path: `App\User`
    pub path: QualifiedName,
    /// Optional alias: as U
    pub alias: Option<String>,
    /// Import type
    pub kind: UseKind,
    #[allow(dead_code)]
    pub span: Span,
}

impl UseItem {
    /// Get the name this import is available as (alias or last segment)
    #[must_use]
    pub fn imported_name(&self) -> &str {
        self.alias
            .as_deref()
            .unwrap_or_else(|| self.path.last().unwrap_or(""))
    }
}

/// Top-level use declaration: `use App\User, App\Post as P;`
#[derive(Debug, Clone)]
pub struct UseDecl {
    pub items: Vec<UseItem>,
    #[allow(dead_code)]
    pub span: Span,
}

/// Class-level use for traits (preparation for future)
#[derive(Debug, Clone)]
pub struct TraitUse {
    pub traits: Vec<QualifiedName>,
    #[allow(dead_code)]
    pub span: Span,
}

/// A compilation unit (single file)
#[derive(Debug, Clone, Default)]
pub struct CompilationUnit {
    /// Optional namespace declaration
    pub namespace: Option<NamespaceDecl>,
    /// Top-level use declarations
    pub uses: Vec<UseDecl>,
    /// Functions defined in this file
    pub functions: Vec<super::Function>,
    /// Classes defined in this file
    pub classes: Vec<super::ClassDef>,
    /// Traits defined in this file
    pub traits: Vec<crate::ast::TraitDef>,
    /// Source file path
    pub file_path: Option<PathBuf>,
}

impl CompilationUnit {
    /// Get the namespace prefix for this unit
    #[must_use]
    #[allow(dead_code)]
    pub fn namespace_prefix(&self) -> Option<&QualifiedName> {
        self.namespace.as_ref().map(|ns| &ns.name)
    }

    /// Qualify a name with this unit's namespace
    #[must_use]
    #[allow(dead_code)]
    pub fn qualify_name(&self, name: &str, span: Span) -> QualifiedName {
        self.namespace.as_ref().map_or_else(
            || QualifiedName::simple(name.to_string(), span),
            |ns| {
                let mut segments = ns.name.segments.clone();
                segments.push(name.to_string());
                QualifiedName::new(segments, false, span)
            },
        )
    }
}
