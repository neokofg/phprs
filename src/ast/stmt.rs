use super::{ClassDef, CompilationUnit, Expr, Span, TraitDef, Type};

/// A complete program (all files combined)
#[derive(Debug, Clone)]
pub struct Program {
    /// All compilation units
    pub units: Vec<CompilationUnit>,
    /// All resolved functions (from all units)
    pub functions: Vec<Function>,
    /// All resolved classes (from all units)
    pub classes: Vec<ClassDef>,
    /// All resolved traits (from all units)
    pub traits: Vec<TraitDef>,
}

impl Program {
    /// Create a program from a single compilation unit (backwards compatibility)
    #[must_use]
    pub fn from_unit(unit: CompilationUnit) -> Self {
        Self {
            functions: unit.functions.clone(),
            classes: unit.classes.clone(),
            traits: unit.traits.clone(),
            units: vec![unit],
        }
    }

    /// Create an empty program
    #[must_use]
    #[allow(dead_code)]
    pub const fn empty() -> Self {
        Self {
            units: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            traits: Vec::new(),
        }
    }
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub is_ref: bool,
    pub span: Span,
}

/// Statement node
#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    #[must_use]
    pub const fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Statement kinds
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// Variable declaration with optional type: $x: int = 42;
    Let {
        name: String,
        ty: Option<Type>,
        init: Expr,
    },

    /// Assignment: $x = expr;
    Assign { target: String, value: Expr },

    /// Compound assignment: $x += expr;
    CompoundAssign {
        target: String,
        op: super::BinaryOp,
        value: Expr,
    },

    /// Expression statement: `foo()`;
    Expr(Expr),

    /// Return statement: return expr;
    Return(Option<Expr>),

    /// If statement
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },

    /// While loop
    While { condition: Expr, body: Vec<Stmt> },

    /// For loop: for ($i = 0; $i < 10; $i++) { ... }
    For {
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Stmt>,
    },

    /// Echo statement: echo $x;
    Echo(Vec<Expr>),

    /// Block of statements
    Block(Vec<Stmt>),
}
