use std::fmt::Display;

use logger::Color;

#[derive(Debug, PartialEq, PartialOrd)]
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
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Misc
    Equals,
}

impl TokenType {
    pub fn as_color(&self) -> Option<Color> {
        match self {
            TokenType::String(_) => Some(Color::BrightGreen),
            TokenType::Number(_) => Some(Color::Yellow),
            TokenType::Float(_) => Some(Color::Yellow),
            TokenType::Bool(_) => Some(Color::Yellow),
            TokenType::Identifier(_) => None,
            TokenType::Let => Some(Color::Blue),
            TokenType::Import => Some(Color::Blue),
            TokenType::Plus => None,
            TokenType::Minus => None,
            TokenType::Multiply => None,
            TokenType::Slash => None,
            TokenType::LParen => None,
            TokenType::RParen => None,
            TokenType::LBracket => None,
            TokenType::RBracket => None,
            TokenType::LBrace => None,
            TokenType::RBrace => None,
            TokenType::Equals => None,
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::String(v) => write!(f, "\"{v}\""),
            TokenType::Number(v) => write!(f, "{v}"),
            TokenType::Float(v) => write!(f, "{v}"),
            TokenType::Bool(v) => write!(f, "{v}"),
            TokenType::Identifier(v) => write!(f, "{v}"),
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
        }
    }
}

#[derive(Debug)]
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
    pub fn new(token_type: TokenType, line: usize, column: usize, len: usize) -> Self {
        Self {
            token_type,
            line,
            column,
            len,
        }
    }
}
