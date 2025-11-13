use crate::parser::{
    ast::types::{Error, ErrorKind},
    lexer::Lexer,
};
use miette::NamedSource;

pub mod ast;
pub mod lexer;

/// # Errors
/// This function will return an error if either the tokenization or AST generation fails.
pub fn parse(source: &NamedSource<String>) -> ast::ExprResult {
    let mut lexer = Lexer::new(source.clone());
    let tokens = lexer.tokenize().map_err(|err| {
        let span = err.span;
        Error::new(ErrorKind::TokenizationError(err), source.clone(), span)
    })?;

    ast::Parser::new(tokens, source.clone()).parse()
}
