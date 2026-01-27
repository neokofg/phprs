use logos::Logos;

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n]+")]
pub enum TokenKind {
    // PHP open tag
    #[token("<?php")]
    PhpOpen,

    // Keywords
    #[token("function")]
    Fn,

    // Arrow function keyword (short closures)
    #[token("fn")]
    FnArrow,

    #[token("return")]
    Return,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("echo")]
    Echo,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("null")]
    Null,

    // OOP Keywords
    #[token("class")]
    Class,

    #[token("new")]
    New,

    #[token("public")]
    Public,

    #[token("private")]
    Private,

    #[token("protected")]
    Protected,

    #[token("extends")]
    Extends,

    #[token("implements")]
    Implements,

    #[token("interface")]
    Interface,

    #[token("abstract")]
    Abstract,

    #[token("static")]
    Static,

    #[token("final")]
    Final,

    #[token("const")]
    Const,

    #[token("$this")]
    This,

    #[token("self")]
    SelfKw,

    #[token("parent")]
    Parent,

    #[token("trait")]
    Trait,

    #[token("use")]
    Use,

    // Exception keywords
    #[token("try")]
    Try,

    #[token("catch")]
    Catch,

    #[token("finally")]
    Finally,

    #[token("throw")]
    Throw,

    #[token("namespace")]
    Namespace,

    #[token("as")]
    As,

    #[token("\\")]
    Backslash,

    // Types
    #[token("int")]
    TypeInt,

    #[token("float")]
    TypeFloat,

    #[token("string")]
    TypeString,

    #[token("bool")]
    TypeBool,

    #[token("void")]
    TypeVoid,

    // Literals
    #[regex(r"[0-9]+", priority = 2)]
    Integer,

    #[regex(r"[0-9]+\.[0-9]+")]
    Float,

    #[regex(r#""([^"\\]|\\.)*""#)]
    String,

    #[regex(r#"'([^'\\]|\\.)*'"#)]
    StringSingle,

    // Identifiers and variables
    #[regex(r"\$[a-zA-Z_][a-zA-Z0-9_]*")]
    Variable,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // Operators
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("=")]
    Assign,

    #[token("==")]
    Eq,

    #[token("!=")]
    Ne,

    #[token("<")]
    Lt,

    #[token("<=")]
    Le,

    #[token(">")]
    Gt,

    #[token(">=")]
    Ge,

    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token("|")]
    Pipe,

    #[token("!")]
    Not,

    #[token("++")]
    PlusPlus,

    #[token("--")]
    MinusMinus,

    #[token("+=")]
    PlusAssign,

    #[token("-=")]
    MinusAssign,

    #[token("*=")]
    StarAssign,

    #[token("/=")]
    SlashAssign,

    #[token(".")]
    Dot,

    #[token("..")]
    DotDot,

    #[token("->")]
    Arrow,

    #[token("&")]
    Ampersand,

    #[token("::")]
    DoubleColon,

    #[token("=>")]
    FatArrow,

    // Attributes
    #[token("#[")]
    HashBracket,

    // Delimiters
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    // Comments
    // Note: #[^\[] ensures # comments don't capture #[ (attribute start)
    #[regex(r"//[^\n]*")]
    #[regex(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")]
    #[regex(r"#[^\[\n][^\n]*")]
    #[regex(r"#\n?", priority = 0)]
    Comment,

    // EOF marker (added manually)
    Eof,
}

#[allow(dead_code)]
impl TokenKind {
    #[must_use]
    pub const fn is_type_keyword(self) -> bool {
        matches!(
            self,
            Self::TypeInt | Self::TypeFloat | Self::TypeString | Self::TypeBool | Self::TypeVoid
        )
    }

    #[must_use]
    pub const fn is_binary_operator(self) -> bool {
        matches!(
            self,
            Self::Plus
                | Self::Minus
                | Self::Star
                | Self::Slash
                | Self::Percent
                | Self::Eq
                | Self::Ne
                | Self::Lt
                | Self::Le
                | Self::Gt
                | Self::Ge
                | Self::And
                | Self::Or
                | Self::Dot
        )
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Token {
    pub kind: TokenKind,
    pub span: (usize, usize),
    pub lexeme: String,
}
