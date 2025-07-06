use super::err;
use crate::parser::tokenizer::{self, types::TokenType};
use logger::{location::Section, Log, LogLevel};
use std::{collections::BTreeMap, fmt::Display, io, path::PathBuf};

#[derive(Debug, PartialEq, Clone)]
pub struct Statement {
    pub statement_type: StatementType,
    pub section: Section,
}

impl Statement {
    pub fn new(statement_type: StatementType, section: Section) -> Self {
        Self {
            statement_type,
            section,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementType {
    Let { name: String, value: Expr },
    Expr(Expr),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub expr_type: ExprType,
    pub section: Section,
}

impl Expr {
    pub fn new(expr_type: ExprType, section: Section) -> Self {
        Self { expr_type, section }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprType {
    Not(Box<Expr>),
    Literal(Literal),
    Identifier(String),
    /// ident, index
    ArrayIndex(String, usize),
    FieldAccess {
        base: Box<Expr>,
        path: Vec<Expr>,
    },
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
    FnDecl {
        args: Vec<String>,
        body: Vec<Statement>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Null,
    Int(isize),
    Float(f64),
    Boolean(bool),
    String(String),
    InterpolatedString(Vec<Expr>),
    Path(PathBuf),
    Array(Vec<Expr>),
    Object(BTreeMap<String, Expr>),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum BinaryOperator {
    // Math Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,

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
}

impl BinaryOperator {
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Plus | Self::Minus => 1,
            Self::Multiply | Self::Divide => 2,
            _ => 0,
        }
    }

    /// # Errors
    /// This function will return an error if the token type is not a binary operator.
    pub fn from_token(token_type: TokenType) -> Result<Self, Box<Error>> {
        match token_type {
            // Math Operators
            TokenType::Plus => Ok(Self::Plus),
            TokenType::Minus => Ok(Self::Minus),
            TokenType::Multiply => Ok(Self::Multiply),
            TokenType::Slash => Ok(Self::Divide),
            TokenType::Modulo => Ok(Self::Modulo),

            // Logic Operators
            TokenType::Eq => Ok(Self::Eq),
            TokenType::NotEq => Ok(Self::NotEq),
            TokenType::Gt => Ok(Self::Gt),
            TokenType::GtEq => Ok(Self::GtEq),
            TokenType::Lt => Ok(Self::Lt),
            TokenType::LtEq => Ok(Self::LtEq),
            TokenType::And => Ok(Self::And),
            TokenType::Or => Ok(Self::Or),

            _ => err!(UnexpectedToken(token_type)),
        }
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                // Math Operators
                Self::Plus => "+",
                Self::Minus => "-",
                Self::Multiply => "*",
                Self::Divide => "/",
                Self::Modulo => "%",

                // Logic Operators
                Self::Eq => "==",
                Self::NotEq => "!=",
                Self::Gt => ">",
                Self::GtEq => ">=",
                Self::Lt => "<",
                Self::LtEq => "<=",
                Self::And => "&&",
                Self::Or => "||",
            }
        )
    }
}

pub type Error = crate::Error<ErrorType>;

#[derive(Debug)]
pub enum ErrorType {
    /// When the right side of a binary operation is missing.
    MissingRightSide,

    /// When a binary operator is expected but another token type was found.
    InvalidBinaryOperator(TokenType),

    /// When an array index is negative.
    NegativeArrayIndex,

    /// When a colon is found between object key-value pairs.
    UnexpectedColonInObjectKV,

    /// When no seperator is found between object key-value pairs.
    ExpectedSeperatorInObjectKV,

    /// When no identifier is found after the 'let' keyword.
    NoIdentifierAfterLet,

    /// When a specific token type is expected but another token type was found.
    ExpectedTokenGot(TokenType, TokenType),
    /// When a token is expected but no tokens are left.
    ExpectedOneOfTokens(Vec<TokenType>),

    /// Very rare edge case in which a token can not be parsed by the AST regardless of the context its in.
    UnexpectedToken(TokenType),

    /// Occurs when no tokens are left.
    NoTokensLeft,

    /// An error that may occur when parsing interpolated strings.
    TokenizationError(tokenizer::types::Error),
    IOError(io::Error),
}

impl PartialEq for ErrorType {
    fn eq(&self, other: &Self) -> bool {
        use ErrorType as E;

        match (self, other) {
            (E::InvalidBinaryOperator(r_token_type), E::InvalidBinaryOperator(l_token_type))
                if r_token_type == l_token_type =>
            {
                true
            }
            (E::NegativeArrayIndex, E::NegativeArrayIndex)
            | (E::NoTokensLeft, E::NoTokensLeft)
            | (E::UnexpectedColonInObjectKV, E::UnexpectedColonInObjectKV)
            | (E::MissingRightSide, E::MissingRightSide)
            | (E::ExpectedSeperatorInObjectKV, E::ExpectedSeperatorInObjectKV)
            | (E::NoIdentifierAfterLet, E::NoIdentifierAfterLet) => true,
            (E::ExpectedTokenGot(r_expected, r_got), E::ExpectedTokenGot(l_expected, l_got))
                if r_expected == l_expected && r_got == l_got =>
            {
                true
            }
            (E::ExpectedOneOfTokens(r_expected), E::ExpectedOneOfTokens(l_expected))
                if r_expected == l_expected =>
            {
                true
            }
            (E::UnexpectedToken(r_token), E::UnexpectedToken(l_token)) if r_token == l_token => {
                true
            }
            (E::TokenizationError(r_error), E::TokenizationError(l_error))
                if r_error == l_error =>
            {
                true
            }

            _ => false,
        }
    }
}

impl From<Error> for Log {
    fn from(value: Error) -> Self {
        let mut log = Log {
            level: LogLevel::Error,
            message: "Unknown AST parsing error".into(),
            location: value.location,
            hint: None,
        };

        #[allow(clippy::single_match)]
        match value.error_type {
            ErrorType::UnexpectedColonInObjectKV => {
                log.hint = Some("Use '=' instead".into());
            }
            ErrorType::TokenizationError(error) => return Self::from(error),
            _ => {}
        }

        log.message(match value.error_type {
            ErrorType::MissingRightSide => "Missing right side of binary operation".into(),
            ErrorType::InvalidBinaryOperator(token) => format!("Invalid binary operator: {token}"),
            ErrorType::NegativeArrayIndex => "Can not index array with negative index".into(),
            ErrorType::UnexpectedColonInObjectKV => {
                "Unexpected ':' between object key-value pairs".into()
            }
            ErrorType::ExpectedSeperatorInObjectKV => "Expected ':' or '=' after object key".into(),
            ErrorType::NoIdentifierAfterLet => "Expected identifier after 'let' keyword".into(),
            ErrorType::ExpectedTokenGot(expected, found) => {
                format!("Expected token '{expected}' found '{found}'")
            }
            ErrorType::ExpectedOneOfTokens(expected) => {
                if expected.len() == 1 {
                    format!(
                        "Expected token '{}', but no tokens are left",
                        #[allow(clippy::unwrap_used, reason = "length checked before")]
                        expected.first().unwrap()
                    )
                } else {
                    format!(
                        "Expected one of the following tokens: {}, but no tokens are left",
                        expected
                            .into_iter()
                            .map(|token| format!("'{token}'"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            ErrorType::UnexpectedToken(token) => format!("Unexpected token: {token}"),
            ErrorType::NoTokensLeft => "No tokens left".into(),
            ErrorType::TokenizationError(error) => format!("Tokenization error: {error}"),
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
