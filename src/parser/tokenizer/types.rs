#![allow(clippy::struct_field_names)]

use logger::Color;
use std::fmt::Display;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum TokenType {
    // Literals
    String(String),
    Number(i64),
    Float(f64),
    Bool(bool),

    // Identifiers
    Identifier(String),

    // Keywords
    Let,
    Import,

    // Operators
    Plus,
    Minus,
    Multiply,
    Slash,

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
}

impl TokenType {
    #[must_use]
    pub fn as_color(&self) -> Option<Color> {
        match self {
            TokenType::String(_) => Some(Color::BrightGreen),
            TokenType::Number(_) | TokenType::Float(_) | TokenType::Bool(_) => Some(Color::Yellow),
            TokenType::Let | TokenType::Import => Some(Color::Blue),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_binary_operator(&self) -> bool {
        matches!(
            self,
            TokenType::Plus | TokenType::Minus | TokenType::Multiply | TokenType::Slash | TokenType::Equals
        )
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::String(v) => write!(f, "\"{v}\""),
            TokenType::Number(v) => write!(f, "{v}"),
            TokenType::Float(v) => write!(f, "{v}"),
            TokenType::Bool(v) => write!(f, "{v}"),
            TokenType::Identifier(v) => write!(f, "identifier: {v}"),
            TokenType::Let => write!(f, "let"),
            TokenType::Import => write!(f, "import"),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Multiply => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::LParen => write!(f, "("),
            TokenType::RParen => write!(f, ")"),
            TokenType::LBracket => write!(f, "["),
            TokenType::RBracket => write!(f, "]"),
            TokenType::LBrace => write!(f, "{{"),
            TokenType::RBrace => write!(f, "}}"),
            TokenType::Equals => write!(f, "="),
            TokenType::Comma => write!(f, ","),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
    pub len: usize,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.token_type == other.token_type
    }
}

impl Token {
    #[must_use]
    pub fn new(token_type: TokenType, line: usize, column: usize, len: usize) -> Self {
        Self {
            token_type,
            line,
            column,
            len,
        }
    }
}
