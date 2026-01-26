/// Type representation in the AST
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// 64-bit signed integer
    Int,
    /// 64-bit floating point
    Float,
    /// Boolean
    Bool,
    /// Heap-allocated string
    String,
    /// No return value
    Void,
    /// Reference type (borrow)
    Ref(Box<Type>),
    /// Mutable reference type
    RefMut(Box<Type>),
    /// Unknown type (to be inferred)
    Unknown,
}

impl Type {
    /// Returns true if this type implements Copy semantics
    #[must_use]
    pub const fn is_copy(&self) -> bool {
        matches!(self, Self::Int | Self::Float | Self::Bool)
    }

    /// Returns true if this type is a reference
    #[must_use]
    pub const fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_) | Self::RefMut(_))
    }

    /// Returns the inner type if this is a reference
    #[must_use]
    pub fn inner_type(&self) -> Option<&Type> {
        match self {
            Self::Ref(inner) | Self::RefMut(inner) => Some(inner),
            _ => None,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::Bool => write!(f, "bool"),
            Self::String => write!(f, "string"),
            Self::Void => write!(f, "void"),
            Self::Ref(inner) => write!(f, "&{inner}"),
            Self::RefMut(inner) => write!(f, "&mut {inner}"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}
