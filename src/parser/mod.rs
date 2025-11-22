use crate::{
    Source,
    parser::{
        ast::types::error::{Error, ErrorKind},
        lexer::Lexer,
    },
};

pub mod ast;
pub mod lexer;

/// # Errors
/// This function will return an error if either the tokenization or AST generation fails.
pub fn parse(source: &Source) -> ast::ExprResult {
    let mut lexer = Lexer::new(source.clone());
    let tokens = lexer.tokenize().map_err(|err| {
        let span = err.span;
        Error::new(ErrorKind::TokenizationError(err), source.clone(), span)
    })?;

    ast::Parser::new(tokens, source.clone()).parse()
}
