use logos::Logos;

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n]+")]
pub enum TokenKind {
    // PHP open tag
    #[token("<?php")]
    PhpOpen,

    // Keywords
    #[token("fn")]
    Fn,

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
    #[regex(r"//[^\n]*")]
    #[regex(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")]
    #[regex(r"#[^\n]*")]
    Comment,

    // EOF marker (added manually)
    Eof,
}

impl TokenKind {
    pub fn is_type_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::TypeInt
                | TokenKind::TypeFloat
                | TokenKind::TypeString
                | TokenKind::TypeBool
                | TokenKind::TypeVoid
        )
    }

    pub fn is_binary_operator(&self) -> bool {
        matches!(
            self,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::Eq
                | TokenKind::Ne
                | TokenKind::Lt
                | TokenKind::Le
                | TokenKind::Gt
                | TokenKind::Ge
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Dot
        )
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: (usize, usize),
    pub lexeme: String,
}
