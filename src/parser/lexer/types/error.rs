use miette::Diagnostic;
use std::{
    io,
    num::{ParseFloatError, ParseIntError},
};
use thiserror::Error;

pub type Error = crate::Error<ErrorKind>;

#[derive(Error, Diagnostic, Debug)]
#[error("Lexer error")]
pub enum ErrorKind {
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),

    #[error("Unclosed string literal")]
    #[diagnostic(code(tl::parser::lexer::unclosed_string))]
    UnclosedString,
    #[error("Unclosed interpolation")]
    #[diagnostic(code(tl::parser::lexer::unclosed_interpolation))]
    UnclosedInterpolation,

    #[error("Unexpected token")]
    #[diagnostic(code(tl::parser::lexer::unexpected_token))]
    UnexpectedToken,

    #[error(transparent)]
    IO(#[from] io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(&self.kind) == std::mem::discriminant(&other.kind)
    }
}
