use crate::parser::tokenizer::{self, types::TokenType};
use logger::{Location, Log, LogLevel};
use std::{collections::BTreeMap, fmt::Display, io, ops::RangeInclusive};

#[derive(Debug, PartialEq, Clone)]
pub struct Statement {
    pub statement_type: StatementType,
    pub line: usize,
    pub cols: RangeInclusive<usize>,
}

impl Statement {
    #[must_use]
    pub fn new(statement_type: StatementType, line: usize, cols: RangeInclusive<usize>) -> Self {
        Self {
            statement_type,
            line,
            cols,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementType {
    Let { name: String, value: Expr },
    Struct { name: String, fields: Vec<String> },
    Expr(Expr),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub expr_type: ExprType,
    pub line: usize,
    pub cols: RangeInclusive<usize>,
}

impl Expr {
    #[must_use]
    pub fn new(expr_type: ExprType, line: usize, cols: RangeInclusive<usize>) -> Self {
        Self { expr_type, line, cols }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprType {
    Not(Box<Expr>),
    Literal(Literal),
    Identifier(String),
    DotAccess(Vec<String>),
    // TODO: Logic operators
    // TODO: For binary operations, make sure to support order of operations with parenthesis
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    InterpolatedString(Vec<Expr>),
    Array(Vec<Expr>),
    Object(BTreeMap<String, Expr>),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl BinaryOperator {
    #[must_use]
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOperator::Plus | BinaryOperator::Minus => 1,
            BinaryOperator::Multiply | BinaryOperator::Divide => 2,
        }
    }

    /// # Errors
    /// This function will return an error if the token type is not a binary operator.
    pub fn from_token(token_type: TokenType) -> Result<Self, Box<Error>> {
        match token_type {
            TokenType::Plus => Ok(BinaryOperator::Plus),
            TokenType::Minus => Ok(BinaryOperator::Minus),
            TokenType::Multiply => Ok(BinaryOperator::Multiply),
            TokenType::Slash => Ok(BinaryOperator::Divide),
            _ => Err(Box::new(Error::new(ErrorType::UnexpectedToken(token_type), None))),
        }
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Plus => write!(f, "+"),
            BinaryOperator::Minus => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
        }
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
    /// When a binary operator is expected but another token type was found.
    InvalidBinaryOperator(TokenType),

    /// When no seperator is found between object key-value pairs.
    ExpectedSeperatorInObjectKV,

    /// When no identifier is found after the 'let' keyword.
    NoIdentifierAfterLet,

    /// When a specific token type is expected but another token type was found.
    ExpectedTokenGot(TokenType, TokenType),
    /// When a token is expected but no tokens are left.
    ExpectedToken(TokenType),

    /// Very rare edge case in which a token can not be parsed by the AST regardless of the context its in.
    UnexpectedToken(TokenType),

    /// An error that may occur when parsing interpolated strings.
    TokenizationError(tokenizer::types::Error),
    IOError(io::Error),
}

impl From<Error> for Log {
    fn from(value: Error) -> Self {
        let log = Log {
            level: LogLevel::Error,
            message: "Unknown AST parsing error".into(),
            location: value.location,
            hint: None,
        };

        log.message(match value.error_type {
            ErrorType::InvalidBinaryOperator(token) => format!("Invalid binary operator: {token}"),
            ErrorType::ExpectedSeperatorInObjectKV => "Expected ':' or '=' after object key".to_string(),
            ErrorType::NoIdentifierAfterLet => "Expected identifier after 'let' keyword".to_string(),
            ErrorType::TokenizationError(error) => format!("Tokenization error: {error}"),
            ErrorType::ExpectedTokenGot(expected, found) => format!("Expected token '{expected}' found '{found}'"),
            ErrorType::ExpectedToken(expected) => format!("Expected token '{expected}', but no tokens are left"),
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

impl From<tokenizer::types::Error> for Error {
    fn from(value: tokenizer::types::Error) -> Self {
        Self::new(ErrorType::TokenizationError(value), None)
    }
}
