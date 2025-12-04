use crate::parser::lexer::types::token::TokenKind;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

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

    #[error("Invalid expression for object key")]
    #[diagnostic(code(tl::parser::ast::object::invalid_key))]
    #[diagnostic(help(
        "The only allowed expressions for an object key are identifiers, strings, interpolated strings, or member accesses (nested objects)"
    ))]
    InvalidObjectKey,

    #[error("Expected '=' after object key")]
    #[diagnostic(code(tl::parser::ast::expected_separator))]
    ExpectedSeparatorInObjectKV,

    #[error("Expected identifier after dot in 'MemberAccess' expression")]
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
    TokenizationError(#[from] super::super::super::lexer::types::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(&self.kind) == std::mem::discriminant(&other.kind)
    }
}
