use super::{Param, Span, Stmt, Type};

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
    #[allow(dead_code)]
    pub fn with_type(mut self, ty: Type) -> Self {
        self.ty = Some(ty);
        self
    }
}

/// Expression kinds
#[derive(Debug, Clone)]
#[allow(dead_code)]
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

    // === OOP Expressions ===
    /// Object instantiation: new ClassName($args)
    New { class_name: String, args: Vec<Expr> },

    /// $this reference
    This,

    /// Property access: $obj->property
    PropertyAccess { object: Box<Expr>, property: String },

    /// Method call: $obj->method($args)
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    /// Static property access: `ClassName::$property`
    StaticPropertyAccess {
        class_name: String,
        property: String,
    },

    /// Static method call: `ClassName::method($args)`
    StaticMethodCall {
        class_name: String,
        method: String,
        args: Vec<Expr>,
    },

    /// Property assignment: $obj->property = expr
    PropertyAssign {
        object: Box<Expr>,
        property: String,
        value: Box<Expr>,
    },

    /// Static property assignment: `ClassName::$property` = expr
    StaticPropertyAssign {
        class_name: String,
        property: String,
        value: Box<Expr>,
    },

    /// Array literal: [1, 2, 3] or ["a" => 1, "b" => 2]
    ArrayLit(Vec<ArrayElement>),

    /// Array access: `$arr[0]` or `$arr["key"]`
    ArrayAccess { array: Box<Expr>, index: Box<Expr> },

    /// Closure/anonymous function
    /// Short: fn($x) => $x + 1
    /// Full: function($x) use ($y) { return $x + $y; }
    Closure {
        params: Vec<Param>,
        return_type: Option<Type>,
        body: ClosureBody,
        captures: Vec<Capture>,
        is_static: bool,
    },

    /// Closure call: $closure($args)
    ClosureCall { closure: Box<Expr>, args: Vec<Expr> },
}

/// Array element (for array literals)
#[derive(Debug, Clone)]
pub struct ArrayElement {
    pub key: Option<Expr>,
    pub value: Expr,
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
    #[allow(dead_code, clippy::unused_self)]
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

/// Closure body - either an expression (arrow) or block
#[derive(Debug, Clone)]
pub enum ClosureBody {
    /// Arrow expression: fn($x) => $x + 1
    Arrow(Box<Expr>),
    /// Block body: function($x) { return $x + 1; }
    Block(Vec<Stmt>),
}

/// Variable capture in closure
#[derive(Debug, Clone)]
pub struct Capture {
    /// Variable name (without $)
    pub name: String,
    /// Whether captured by reference (&$x)
    pub by_ref: bool,
    pub span: Span,
}
