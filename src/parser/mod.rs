use crate::source::Source;
use ast::StatementResult;

pub mod ast;
pub mod tokenizer;

/// # Errors
/// This function will return an error if either the tokenization or AST generation fails.
pub fn parse(source: Source) -> StatementResult {
    let mut tokenizer = tokenizer::Parser::new(&source);
    let tokens = tokenizer.tokenize()?;
    let mut ast = ast::Parser::new(tokens, source);

    ast.parse()
}
