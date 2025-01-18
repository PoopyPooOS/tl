use logger::{Log, LogLevel};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
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
    Null,

    // Identifiers
    Identifier(String),

    // Keywords
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

                // Logical Operator
                | Self::Dot
        )
    }

    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Int(_) | Self::Float(_))
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Literals
            Self::InterpolatedString(_) => write!(f, "interpolated string"),
            Self::String(v) => write!(f, "\"{v}\""),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Bool(v) => write!(f, "{v}"),
            Self::Null => write!(f, "null"),

            // Identifiers
            #[cfg(debug_assertions)]
            Self::Identifier(v) => write!(f, "identifier: {v}"),
            #[cfg(not(debug_assertions))]
            Self::Identifier(v) => write!(f, "{v}"),

            // Keywords
            Self::Let => write!(f, "let"),

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

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub cols: RangeInclusive<usize>,
}

impl Token {
    #[must_use]
    pub fn new(token_type: TokenType, line: usize, cols: RangeInclusive<usize>) -> Self {
        Self {
            token_type,
            line,
            cols,
        }
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

pub type Error = crate::Error<ErrorType>;

#[derive(Debug)]
pub enum ErrorType {
    // Number errors
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),

    // String errors
    UnclosedString,
    UnclosedInterpolation,

    UnexpectedToken(char),
    IOError(io::Error),
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
            // Number errors
            ErrorType::ParseIntError(error) => format!("Failed to parse int: {error}"),
            ErrorType::ParseFloatError(error) => format!("Failed to parse float: {error}"),

            // String errors
            ErrorType::UnclosedString => "Unclosed string literal".to_string(),
            ErrorType::UnclosedInterpolation => "Unclosed string interpolation".to_string(),

            ErrorType::UnexpectedToken(token) => format!("Unexpected token: {token}"),
            ErrorType::IOError(error) => format!("IO error: {error}"),
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
