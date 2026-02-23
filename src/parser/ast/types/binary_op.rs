use crate::{
    Source,
    parser::{
        ast::types::{Error, ErrorKind},
        lexer::types::token::TokenKind,
    },
};
use std::fmt::Display;

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
            // Logical OR (lowest)
            Self::Or => 1,
            // Logical AND
            Self::And => 2,
            // Comparison operators
            Self::Eq | Self::NotEq | Self::Gt | Self::GtEq | Self::Lt | Self::LtEq => 3,
            // Addition and subtraction
            Self::Plus | Self::Minus => 4,
            // Multiplication, division, and modulo (highest)
            Self::Multiply | Self::Divide | Self::Modulo => 5,
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
                    Source::text("kind"),
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
