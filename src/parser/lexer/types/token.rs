use miette::SourceSpan;
use std::{
    fmt::{self, Display},
    path::PathBuf,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

impl Token {
    pub fn new(kind: TokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // Literals
    String(String),
    InterpolatedString(Vec<Token>),
    Path(PathBuf),
    InterpolatedPath(Vec<Token>),
    Int(isize),
    Float(f64),
    Bool(bool),
    Null,

    // Identifiers
    Identifier(String),

    // Keywords
    With,
    Let,
    In,

    // Logic Operators
    /// ==
    Eq,
    /// !=
    NotEq,
    /// >
    Gt,
    /// >=
    GtEq,
    /// <
    Lt,
    /// <=
    LtEq,
    /// &&
    And,
    /// ||
    Or,

    // Unary Operators
    Not,

    // Binary Operators
    Plus,
    Minus,
    Multiply,
    Slash,
    Modulo,

    // Brackets
    /// (
    LParen,
    /// )
    RParen,
    /// [
    LBracket,
    /// ]
    RBracket,
    /// {
    LBrace,
    /// }
    RBrace,

    // Misc
    Equals,
    Comma,
    Colon,
    Dot,
}

impl TokenKind {
    pub fn is_binary_operator(&self) -> bool {
        matches!(
            self,
            // Math Operators
            Self::Plus
                | Self::Minus
                | Self::Multiply
                | Self::Slash
                | Self::Modulo

                // Logic Operators
                | Self::Eq
                | Self::NotEq
                | Self::Gt
                | Self::GtEq
                | Self::Lt
                | Self::LtEq
                | Self::And
                | Self::Or
        )
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Int(_) | Self::Float(_))
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Literals
            // TODO: Handle interpolated strings/paths
            Self::InterpolatedString(_) => write!(f, "interpolated string"),
            Self::String(v) => write!(f, "\"{v}\""),
            Self::InterpolatedPath(_) => write!(f, "interpolated path"),
            Self::Path(v) => write!(f, "{}", v.display()),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Bool(v) => write!(f, "{v}"),
            Self::Null => write!(f, "null"),

            // Identifiers
            Self::Identifier(v) => write!(f, "{v}"),

            // Keywords
            Self::With => write!(f, "with"),
            Self::Let => write!(f, "let"),
            Self::In => write!(f, "in"),

            // Logic Operators
            Self::Eq => write!(f, "=="),
            Self::NotEq => write!(f, "!="),
            Self::Gt => write!(f, ">"),
            Self::GtEq => write!(f, ">="),
            Self::Lt => write!(f, "<"),
            Self::LtEq => write!(f, "<="),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::Not => write!(f, "!"),

            // Binary Operators
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Modulo => write!(f, "%"),

            // Brackets
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),

            // Misc Symbols
            Self::Equals => write!(f, "="),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::Dot => write!(f, "."),
        }
    }
}
