//! Standard Library Intrinsics
//!
//! This module provides built-in function declarations that map to runtime intrinsics.
//! Functions are embedded at compile time for zero-cost inclusion.

use crate::ast::{
    Attribute, AttributeArg, Attributes, Expr, ExprKind, Function, Param, Span, Type,
};

/// Get all stdlib function declarations
#[must_use]
pub fn get_stdlib_functions() -> Vec<Function> {
    let mut functions = Vec::new();

    // String functions
    functions.extend(string_functions());

    // Array functions
    functions.extend(array_functions());

    // Math functions
    functions.extend(math_functions());

    // Type functions
    functions.extend(type_functions());

    functions
}

/// Create an intrinsic function declaration
fn intrinsic(
    name: &str,
    runtime_name: &str,
    params: Vec<(&str, Type)>,
    return_type: Type,
) -> Function {
    let span = Span::new(0, 0);

    // Build #[Intrinsic("runtime_name")] attribute
    let intrinsic_attr = Attribute {
        name: "Intrinsic".to_string(),
        args: vec![AttributeArg::Positional(Expr {
            kind: ExprKind::StringLit(runtime_name.to_string()),
            span,
            ty: Some(Type::String),
        })],
        span,
    };

    // Build #[Inline] attribute
    let inline_attr = Attribute {
        name: "Inline".to_string(),
        args: vec![],
        span,
    };

    let attributes = Attributes::with_items(vec![inline_attr, intrinsic_attr]);

    let params = params
        .into_iter()
        .map(|(pname, ty)| Param {
            name: pname.to_string(),
            ty,
            is_ref: false,
            span,
        })
        .collect();

    Function {
        name: name.to_string(),
        params,
        return_type,
        body: vec![], // Intrinsic functions have no body
        attributes,
        span,
    }
}

/// String functions - using runtime C-string wrappers
fn string_functions() -> Vec<Function> {
    vec![
        // Use runtime strlen (via C-string wrapper)
        intrinsic(
            "strlen",
            "rt_cstr_len",
            vec![("s", Type::String)],
            Type::Int,
        ),
        intrinsic(
            "substr",
            "rt_cstr_substr",
            vec![
                ("s", Type::String),
                ("start", Type::Int),
                ("length", Type::Int),
            ],
            Type::String,
        ),
        intrinsic(
            "strpos",
            "rt_cstr_strpos",
            vec![("haystack", Type::String), ("needle", Type::String)],
            Type::Int,
        ),
        intrinsic(
            "strtolower",
            "rt_cstr_tolower",
            vec![("s", Type::String)],
            Type::String,
        ),
        intrinsic(
            "strtoupper",
            "rt_cstr_toupper",
            vec![("s", Type::String)],
            Type::String,
        ),
        intrinsic(
            "trim",
            "rt_cstr_trim",
            vec![("s", Type::String)],
            Type::String,
        ),
        intrinsic(
            "ltrim",
            "rt_cstr_ltrim",
            vec![("s", Type::String)],
            Type::String,
        ),
        intrinsic(
            "rtrim",
            "rt_cstr_rtrim",
            vec![("s", Type::String)],
            Type::String,
        ),
        intrinsic(
            "str_replace",
            "rt_cstr_replace",
            vec![
                ("search", Type::String),
                ("replace", Type::String),
                ("subject", Type::String),
            ],
            Type::String,
        ),
        intrinsic(
            "str_contains",
            "rt_cstr_contains",
            vec![("haystack", Type::String), ("needle", Type::String)],
            Type::Bool,
        ),
        intrinsic(
            "str_starts_with",
            "rt_cstr_starts_with",
            vec![("haystack", Type::String), ("needle", Type::String)],
            Type::Bool,
        ),
        intrinsic(
            "str_ends_with",
            "rt_cstr_ends_with",
            vec![("haystack", Type::String), ("needle", Type::String)],
            Type::Bool,
        ),
        intrinsic(
            "strcmp",
            "rt_cstr_cmp",
            vec![("s1", Type::String), ("s2", Type::String)],
            Type::Int,
        ),
        intrinsic("ord", "rt_cstr_ord", vec![("c", Type::String)], Type::Int),
        intrinsic(
            "chr",
            "rt_cstr_chr",
            vec![("code", Type::Int)],
            Type::String,
        ),
        intrinsic(
            "strrev",
            "rt_cstr_rev",
            vec![("s", Type::String)],
            Type::String,
        ),
        intrinsic(
            "str_repeat",
            "rt_cstr_repeat",
            vec![("s", Type::String), ("count", Type::Int)],
            Type::String,
        ),
    ]
}

/// Array functions - all use our Rust runtime
fn array_functions() -> Vec<Function> {
    vec![
        // Core array functions
        intrinsic(
            "count",
            "rt_count",
            vec![("arr", Type::Array(Box::new(Type::Unknown)))],
            Type::Int,
        ),
        intrinsic(
            "array_sum",
            "rt_array_sum",
            vec![("arr", Type::Array(Box::new(Type::Int)))],
            Type::Int,
        ),
        intrinsic(
            "array_product",
            "rt_array_product",
            vec![("arr", Type::Array(Box::new(Type::Int)))],
            Type::Int,
        ),
        // Search functions
        intrinsic(
            "in_array",
            "rt_in_array",
            vec![
                ("needle", Type::Unknown),
                ("haystack", Type::Array(Box::new(Type::Unknown))),
            ],
            Type::Bool,
        ),
        intrinsic(
            "array_key_exists",
            "rt_array_key_exists_int",
            vec![
                ("key", Type::Int),
                ("arr", Type::Array(Box::new(Type::Unknown))),
            ],
            Type::Bool,
        ),
        intrinsic(
            "array_search",
            "rt_array_search",
            vec![
                ("needle", Type::Unknown),
                ("haystack", Type::Array(Box::new(Type::Unknown))),
            ],
            Type::Int,
        ),
        // Stack/queue operations
        intrinsic(
            "array_pop",
            "rt_array_pop",
            vec![("arr", Type::Array(Box::new(Type::Unknown)))],
            Type::Unknown,
        ),
        intrinsic(
            "array_shift",
            "rt_array_shift",
            vec![("arr", Type::Array(Box::new(Type::Unknown)))],
            Type::Unknown,
        ),
        // Element access
        intrinsic(
            "array_first",
            "rt_array_first",
            vec![("arr", Type::Array(Box::new(Type::Unknown)))],
            Type::Unknown,
        ),
        intrinsic(
            "array_last",
            "rt_array_last",
            vec![("arr", Type::Array(Box::new(Type::Unknown)))],
            Type::Unknown,
        ),
    ]
}

/// Math functions - all use our Rust runtime
fn math_functions() -> Vec<Function> {
    vec![
        // Basic math
        intrinsic("abs", "rt_abs", vec![("n", Type::Int)], Type::Int),
        intrinsic("ceil", "rt_ceil", vec![("n", Type::Float)], Type::Float),
        intrinsic("floor", "rt_floor", vec![("n", Type::Float)], Type::Float),
        intrinsic("round", "rt_round", vec![("n", Type::Float)], Type::Float),
        // Exponential and logarithmic
        intrinsic("sqrt", "rt_sqrt", vec![("n", Type::Float)], Type::Float),
        intrinsic(
            "pow",
            "rt_pow",
            vec![("base", Type::Float), ("exp", Type::Float)],
            Type::Float,
        ),
        intrinsic("exp", "rt_exp", vec![("n", Type::Float)], Type::Float),
        intrinsic("log", "rt_log", vec![("n", Type::Float)], Type::Float),
        intrinsic("log10", "rt_log10", vec![("n", Type::Float)], Type::Float),
        // Trigonometric
        intrinsic("sin", "rt_sin", vec![("n", Type::Float)], Type::Float),
        intrinsic("cos", "rt_cos", vec![("n", Type::Float)], Type::Float),
        intrinsic("tan", "rt_tan", vec![("n", Type::Float)], Type::Float),
        intrinsic("asin", "rt_asin", vec![("n", Type::Float)], Type::Float),
        intrinsic("acos", "rt_acos", vec![("n", Type::Float)], Type::Float),
        intrinsic("atan", "rt_atan", vec![("n", Type::Float)], Type::Float),
        intrinsic(
            "atan2",
            "rt_atan2",
            vec![("y", Type::Float), ("x", Type::Float)],
            Type::Float,
        ),
        // Min/max
        intrinsic(
            "min",
            "rt_min",
            vec![("a", Type::Int), ("b", Type::Int)],
            Type::Int,
        ),
        intrinsic(
            "max",
            "rt_max",
            vec![("a", Type::Int), ("b", Type::Int)],
            Type::Int,
        ),
        // Random
        intrinsic("rand", "rt_rand", vec![], Type::Int),
        intrinsic("srand", "rt_srand", vec![("seed", Type::Int)], Type::Void),
        // Utility
        intrinsic(
            "fmod",
            "rt_fmod",
            vec![("x", Type::Float), ("y", Type::Float)],
            Type::Float,
        ),
        intrinsic(
            "hypot",
            "rt_hypot",
            vec![("x", Type::Float), ("y", Type::Float)],
            Type::Float,
        ),
        intrinsic(
            "is_finite",
            "rt_is_finite",
            vec![("n", Type::Float)],
            Type::Bool,
        ),
        intrinsic("is_nan", "rt_is_nan", vec![("n", Type::Float)], Type::Bool),
        intrinsic(
            "is_infinite",
            "rt_is_infinite",
            vec![("n", Type::Float)],
            Type::Bool,
        ),
    ]
}

/// Type functions
fn type_functions() -> Vec<Function> {
    vec![
        intrinsic(
            "is_null",
            "rt_is_null",
            vec![("v", Type::Unknown)],
            Type::Bool,
        ),
        intrinsic(
            "is_int",
            "rt_is_int",
            vec![("v", Type::Unknown)],
            Type::Bool,
        ),
        intrinsic(
            "is_float",
            "rt_is_float",
            vec![("v", Type::Unknown)],
            Type::Bool,
        ),
        intrinsic(
            "is_string",
            "rt_is_string",
            vec![("v", Type::Unknown)],
            Type::Bool,
        ),
        intrinsic(
            "is_bool",
            "rt_is_bool",
            vec![("v", Type::Unknown)],
            Type::Bool,
        ),
        intrinsic(
            "is_array",
            "rt_is_array",
            vec![("v", Type::Unknown)],
            Type::Bool,
        ),
        intrinsic("intval", "rt_intval", vec![("v", Type::Unknown)], Type::Int),
        intrinsic(
            "floatval",
            "rt_floatval",
            vec![("v", Type::Unknown)],
            Type::Float,
        ),
        intrinsic(
            "strval",
            "rt_strval",
            vec![("v", Type::Unknown)],
            Type::String,
        ),
        intrinsic(
            "boolval",
            "rt_boolval",
            vec![("v", Type::Unknown)],
            Type::Bool,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_loads() {
        let funcs = get_stdlib_functions();
        assert!(!funcs.is_empty());

        // Check strlen exists and maps to runtime
        let strlen = funcs.iter().find(|f| f.name == "strlen");
        assert!(strlen.is_some());

        let strlen = strlen.unwrap();
        assert!(strlen.attributes.get_intrinsic().is_some());
        assert_eq!(strlen.attributes.get_intrinsic().unwrap(), "rt_cstr_len");
    }

    #[test]
    fn test_intrinsic_attribute() {
        let func = intrinsic("test", "rt_test", vec![("x", Type::Int)], Type::Int);
        assert_eq!(func.attributes.get_intrinsic(), Some("rt_test"));
        assert!(func.attributes.is_inline());
    }
}
