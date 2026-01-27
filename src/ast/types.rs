/// Type representation in the AST
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code, clippy::enum_variant_names)]
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
    Ref(Box<Self>),
    /// Mutable reference type
    RefMut(Box<Self>),
    /// Class/object type
    Class(String),
    /// Interface type
    Interface(String),
    /// Nullable type (T or null)
    Nullable(Box<Self>),
    /// Array type
    Array(Box<Self>),
    /// Closure type: `Closure(param_types, return_type)`
    Closure(Vec<Self>, Box<Self>),
    /// Self type (inside class)
    SelfType,
    /// Static type (late static binding)
    StaticType,
    /// Unknown type (to be inferred)
    Unknown,
}

#[allow(dead_code)]
impl Type {
    /// Returns true if this type implements Copy semantics
    #[must_use]
    pub const fn is_copy(&self) -> bool {
        // Unknown is treated as Copy to allow untyped parameters
        matches!(self, Self::Int | Self::Float | Self::Bool | Self::Unknown)
    }

    /// Returns true if this type is a reference
    #[must_use]
    pub const fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_) | Self::RefMut(_))
    }

    /// Returns the inner type if this is a reference
    #[must_use]
    pub fn inner_type(&self) -> Option<&Self> {
        match self {
            Self::Ref(inner) | Self::RefMut(inner) => Some(inner),
            _ => None,
        }
    }

    /// Returns true if this is an object type
    #[must_use]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Class(_) | Self::Interface(_))
    }

    /// Get class name if this is a class type
    #[must_use]
    pub fn class_name(&self) -> Option<&str> {
        match self {
            Self::Class(name) | Self::Interface(name) => Some(name),
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
            Self::Class(name) | Self::Interface(name) => write!(f, "{name}"),
            Self::Nullable(inner) => write!(f, "?{inner}"),
            Self::Array(inner) => write!(f, "array<{inner}>"),
            Self::Closure(params, ret) => {
                write!(f, "Closure(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, "): {ret}")
            }
            Self::SelfType => write!(f, "self"),
            Self::StaticType => write!(f, "static"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}
