use crate::parser::ast::types;
use derive_more::PartialEq;
use miette::{Diagnostic, SourceSpan};
use std::io;
use thiserror::Error;

pub type Error = crate::Error<ErrorKind>;

#[derive(Error, Diagnostic, Debug, PartialEq)]
#[error("Runtime error")]
pub enum ErrorKind {
    #[error("This variable is not in scope")]
    #[diagnostic(code(tl::runtime::expr))]
    VariableNotInScope {
        #[label("Could not find this variable in scope")]
        variable: SourceSpan,
    },

    #[error("Value can not be called as a function")]
    #[diagnostic(code(tl::runtime::call))]
    NotCallable,

    #[error("A value that isn't an object was used in a with expression")]
    #[diagnostic(code(tl::runtime::with_expr))]
    NonObjectInWithExpr,

    #[error("Mismatch in number of function arguments")]
    #[diagnostic(code(tl::runtime::call))]
    ArgsMismatch {
        len: usize,

        #[label("Supposed to have {len} argument{s}", s = if *len == 1 { "" } else { "s" })]
        args: SourceSpan,
    },

    #[error("Index out of bounds")]
    #[diagnostic(code(tl::runtime::expr))]
    IndexOutOfBounds {
        length: usize,

        #[label("Length is {length}")]
        index: SourceSpan,
    },

    #[error("Mismatched types, expected {expected}, got {got}")]
    #[diagnostic(code(tl::runtime::expr))]
    MismatchedTypes { expected: String, got: String },

    #[error(transparent)]
    ParseError(#[from] types::Error),

    #[error(transparent)]
    IOError(
        #[partial_eq(skip)]
        #[from]
        io::Error,
    ),

    #[cfg(feature = "toml")]
    #[error(transparent)]
    TomlParsingError(#[from] toml::de::Error),
}
