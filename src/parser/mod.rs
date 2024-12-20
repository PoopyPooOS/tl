use crate::Source;

pub mod ast;
pub mod tokenizer;

/// # Errors
/// This function will return an error if either the tokenization or AST generation fails.
pub fn parse(source: Source) -> ast::StatementResult {
    let mut tokenizer = tokenizer::Parser::new(&source);
    let tokens = tokenizer.tokenize().map_err(ast::types::Error::from)?;

    ast::Parser::new(tokens, source).parse()
}
