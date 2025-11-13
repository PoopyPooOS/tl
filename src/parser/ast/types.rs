use crate::parser::lexer::{self, types::TokenKind};
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::{collections::BTreeMap, fmt::Display, path::PathBuf};
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SourceSpan,
}

impl Default for Expr {
    fn default() -> Self {
        Self::lit(Literal::Null, SourceSpan::new(0.into(), 0))
    }
}

impl Expr {
    pub const fn new(kind: ExprKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub const fn lit(literal: Literal, span: SourceSpan) -> Self {
        Self::new(ExprKind::Literal(literal), span)
    }

    pub fn ident(ident: impl AsRef<str>, span: SourceSpan) -> Self {
        Self::new(ExprKind::Identifier(ident.as_ref().to_string()), span)
    }

    pub fn boxed(kind: ExprKind, span: SourceSpan) -> Box<Self> {
        Box::new(Self::new(kind, span))
    }

    pub fn boxed_lit(literal: Literal, span: SourceSpan) -> Box<Self> {
        Box::new(Self::lit(literal, span))
    }

    pub fn boxed_ident(ident: impl AsRef<str>, span: SourceSpan) -> Box<Self> {
        Box::new(Self::ident(ident, span))
    }

    pub fn as_ident(&self) -> Option<String> {
        match &self.kind {
            ExprKind::Identifier(ident) => Some(ident.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    Not(Box<Expr>),
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
    ArrayIndex {
        base: Box<Expr>,
        index: usize,
    },
    ObjectAccess {
        base: Box<Expr>,
        field: String,
    },
    FnDecl {
        args: Vec<String>,
        expr: Box<Expr>,
    },
    Call {
        base: Box<Expr>,
        args: Vec<Expr>,
    },
    LetIn {
        bindings: Vec<(String, Expr)>,
        expr: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Null,
    Int(isize),
    Float(f64),
    Bool(bool),
    String(String),
    InterpolatedString(Vec<Expr>),
    Path(PathBuf),
    InterpolatedPath(Vec<Expr>),
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
    pub fn from_token(kind: TokenKind) -> Result<Self, Error> {
        match kind {
            // Math Operators
            TokenKind::Plus => Ok(Self::Plus),
            TokenKind::Minus => Ok(Self::Minus),
            TokenKind::Multiply => Ok(Self::Multiply),
            TokenKind::Slash => Ok(Self::Divide),
            TokenKind::Modulo => Ok(Self::Modulo),

            // Logic Operators
            TokenKind::Eq => Ok(Self::Eq),
            TokenKind::NotEq => Ok(Self::NotEq),
            TokenKind::Gt => Ok(Self::Gt),
            TokenKind::GtEq => Ok(Self::GtEq),
            TokenKind::Lt => Ok(Self::Lt),
            TokenKind::LtEq => Ok(Self::LtEq),
            TokenKind::And => Ok(Self::And),
            TokenKind::Or => Ok(Self::Or),

            _ => {
                let kind = kind.to_string();
                let len = kind.len();

                Err(Error::new(
                    ErrorKind::UnexpectedToken,
                    NamedSource::new("builtin", kind),
                    (0, len).into(),
                ))
            }
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

pub type Error = crate::Error<ErrorKind>;

#[derive(Error, Diagnostic, Debug)]
#[error("Parser error")]
pub enum ErrorKind {
    #[error("Missing right side of binary operation")]
    #[diagnostic(code(tl::parser::ast::missing_right_side))]
    MissingRightSide,

    #[error("Invalid binary operator")]
    #[diagnostic(code(tl::parser::ast::invalid_binary_operator))]
    InvalidBinaryOperator {
        #[label("Operator")]
        operator: SourceSpan,
    },

    #[error("Can not index array with negative index")]
    #[diagnostic(code(tl::parser::ast::array_index))]
    NegativeArrayIndex,

    #[error("Unexpected ':' between object key-value pairs")]
    #[diagnostic(help("Use '=' instead"))]
    #[diagnostic(code(tl::parser::ast::colon_separator))]
    UnexpectedColonInObjectKV,

    #[error("Expected '=' after object key")]
    #[diagnostic(code(tl::parser::ast::expected_separator))]
    ExpectedSeparatorInObjectKV,

    #[error("Expected identifier after dot in 'FieldAccess' expression")]
    #[diagnostic(code(tl::parser::ast::expected_identifier))]
    ExpectedIdentifierAfterDot,

    #[error("Expected {expected} token{found_msg}", found_msg = if let Self::ExpectedToken { found: Some(found), .. } = self { format!(", found '{found}'") } else { String::new() })]
    #[diagnostic(code(tl::parser::ast::expected_token))]
    ExpectedToken {
        expected: String,
        found: Option<TokenKind>,
    },

    #[error("Unexpected token")]
    #[diagnostic(code(tl::parser::ast::unexpected_token))]
    UnexpectedToken,

    #[error("No tokens left")]
    #[diagnostic(code(tl::parser::ast::no_tokens_left))]
    NoTokensLeft,

    #[error(transparent)]
    TokenizationError(#[from] lexer::types::Error),
}
