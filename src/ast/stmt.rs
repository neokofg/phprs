use super::{Expr, Span, Type};

/// A complete program
#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
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

    /// Expression statement: foo();
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
