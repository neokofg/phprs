mod expr;
mod stmt;
mod types;

pub use expr::{BinaryOp, Expr, ExprKind, UnaryOp};
pub use stmt::{Function, Param, Program, Stmt, StmtKind};
pub use types::Type;

/// Span information for error reporting
#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl From<(usize, usize)> for Span {
    fn from((start, end): (usize, usize)) -> Self {
        Self { start, end }
    }
}

impl From<Span> for (usize, usize) {
    fn from(span: Span) -> Self {
        (span.start, span.end)
    }
}
