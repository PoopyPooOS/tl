#![allow(clippy::struct_field_names)]

use logger::{Location, Log, LogLevel};
use std::{
    cmp::Ordering,
    fmt::Display,
    io,
    num::{ParseFloatError, ParseIntError},
    ops::RangeInclusive,
};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum TokenType {
    // Literals
    InterpolatedString(Vec<Token>),
    String(String),
    Int(isize),
    Float(f64),
    Bool(bool),
    // TODO: Make this an enum similar to `Option`, `null` is not going to work well with the type checker.
    Null,

    // Identifiers
    Identifier(String),

    // Keywords
    Struct,
    Enum,
    Let,

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

impl TokenType {
    #[must_use]
    pub fn is_binary_operator(&self) -> bool {
        matches!(
            self,
            // Math Operators
            TokenType::Plus
                | TokenType::Minus
                | TokenType::Multiply
                | TokenType::Slash
                | TokenType::Modulo

                // Logic Operators
                | TokenType::Eq
                | TokenType::NotEq
                | TokenType::Gt
                | TokenType::GtEq
                | TokenType::Lt
                | TokenType::LtEq
                | TokenType::And
                | TokenType::Or
        )
    }

    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self, TokenType::Int(_) | TokenType::Float(_))
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Literals
            TokenType::InterpolatedString(_) => write!(f, "interpolated string"),
            TokenType::String(v) => write!(f, "\"{v}\""),
            TokenType::Int(v) => write!(f, "{v}"),
            TokenType::Float(v) => write!(f, "{v}"),
            TokenType::Bool(v) => write!(f, "{v}"),
            TokenType::Null => write!(f, "null"),

            // Identifiers
            #[cfg(debug_assertions)]
            TokenType::Identifier(v) => write!(f, "identifier: {v}"),
            #[cfg(not(debug_assertions))]
            TokenType::Identifier(v) => write!(f, "{v}"),

            // Keywords
            TokenType::Struct => write!(f, "struct"),
            TokenType::Enum => write!(f, "enum"),
            TokenType::Let => write!(f, "let"),

            // Logic Operators
            TokenType::Eq => write!(f, "=="),
            TokenType::NotEq => write!(f, "!="),
            TokenType::Gt => write!(f, ">"),
            TokenType::GtEq => write!(f, ">="),
            TokenType::Lt => write!(f, "<"),
            TokenType::LtEq => write!(f, "<="),
            TokenType::And => write!(f, "&&"),
            TokenType::Or => write!(f, "||"),
            TokenType::Not => write!(f, "!"),

            // Binary Operators
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Multiply => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Modulo => write!(f, "%"),

            // Brackets
            TokenType::LParen => write!(f, "("),
            TokenType::RParen => write!(f, ")"),
            TokenType::LBracket => write!(f, "["),
            TokenType::RBracket => write!(f, "]"),
            TokenType::LBrace => write!(f, "{{"),
            TokenType::RBrace => write!(f, "}}"),

            // Misc Symbols
            TokenType::Equals => write!(f, "="),
            TokenType::Comma => write!(f, ","),
            TokenType::Colon => write!(f, ":"),
            TokenType::Dot => write!(f, "."),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub cols: RangeInclusive<usize>,
}

impl Token {
    #[must_use]
    pub fn new(token_type: TokenType, line: usize, cols: RangeInclusive<usize>) -> Self {
        Self { token_type, line, cols }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.token_type == other.token_type
    }
}

impl PartialOrd for Token {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.token_type.partial_cmp(&other.token_type)
    }
}

#[derive(Debug)]
pub struct Error {
    error_type: ErrorType,
    location: Option<Location>,
}

impl std::error::Error for Error {
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.error_type)
    }
}

impl Error {
    #[must_use]
    pub fn new(error_type: ErrorType, location: Option<Location>) -> Self {
        Self { error_type, location }
    }
}

#[derive(Debug)]
pub enum ErrorType {
    UnexpectedToken(char),

    // String errors
    UnclosedString,
    UnclosedInterpolation,

    IOError(io::Error),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
}

impl From<Error> for Log {
    fn from(value: Error) -> Self {
        let log = Log {
            level: LogLevel::Error,
            message: "Unknown tokenization error".into(),
            location: value.location,
            hint: None,
        };

        log.message(match value.error_type {
            ErrorType::UnexpectedToken(token) => format!("Unexpected token: {token}"),

            // String errors
            ErrorType::UnclosedString => "Unclosed string literal".to_string(),
            ErrorType::UnclosedInterpolation => "Unclosed string interpolation".to_string(),

            ErrorType::IOError(error) => format!("IO error: {error}"),
            ErrorType::ParseIntError(error) => format!("Parse int error: {error}"),
            ErrorType::ParseFloatError(error) => format!("Parse float error: {error}"),
        })
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::new(ErrorType::IOError(value), None)
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::new(ErrorType::ParseIntError(value), None)
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Self::new(ErrorType::ParseFloatError(value), None)
    }
}
