use super::{Span, Type};

/// Expression node
#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    /// Type annotation or inferred type
    pub ty: Option<Type>,
}

impl Expr {
    #[must_use]
    pub const fn new(kind: ExprKind, span: Span) -> Self {
        Self {
            kind,
            span,
            ty: None,
        }
    }

    #[must_use]
    pub fn with_type(mut self, ty: Type) -> Self {
        self.ty = Some(ty);
        self
    }
}

/// Expression kinds
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// Integer literal: 42
    IntLit(i64),

    /// Float literal: 3.14
    FloatLit(f64),

    /// Boolean literal: true, false
    BoolLit(bool),

    /// String literal: "hello"
    StringLit(String),

    /// Null literal
    Null,

    /// Variable reference: $x
    Variable(String),

    /// Binary operation: $a + $b
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    /// Unary operation: -$x, !$x
    Unary { op: UnaryOp, operand: Box<Expr> },

    /// Function call: foo($a, $b)
    Call { name: String, args: Vec<Expr> },

    /// Reference (borrow): &$x
    Ref(Box<Expr>),

    /// Mutable reference: &mut $x (using &$x in PHP)
    RefMut(Box<Expr>),

    /// Assignment expression: $x = expr
    Assign { target: String, value: Box<Expr> },

    /// Prefix increment/decrement: ++$x, --$x
    PrefixOp { op: UnaryOp, target: String },

    /// Postfix increment/decrement: $x++, $x--
    PostfixOp { op: UnaryOp, target: String },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,

    // String
    Concat,
}

impl BinaryOp {
    /// Returns the precedence of this operator (higher = binds tighter)
    #[must_use]
    pub const fn precedence(self) -> u8 {
        match self {
            Self::Or => 1,
            Self::And => 2,
            Self::Eq | Self::Ne => 3,
            Self::Lt | Self::Le | Self::Gt | Self::Ge => 4,
            Self::Concat => 5,
            Self::Add | Self::Sub => 6,
            Self::Mul | Self::Div | Self::Mod => 7,
        }
    }

    /// Returns true if this operator is left-associative
    #[must_use]
    pub const fn is_left_assoc(self) -> bool {
        true
    }
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::Concat => write!(f, "."),
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Negation: -x
    Neg,
    /// Logical not: !x
    Not,
    /// Increment: ++
    Inc,
    /// Decrement: --
    Dec,
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Neg => write!(f, "-"),
            Self::Not => write!(f, "!"),
            Self::Inc => write!(f, "++"),
            Self::Dec => write!(f, "--"),
        }
    }
}
