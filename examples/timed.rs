use miette::NamedSource;
use std::{collections::HashMap, time::Instant};
use tl::{
    parser::{ast, lexer},
    runtime::Scope,
};

fn main() -> miette::Result<()> {
    let source = NamedSource::new("examples/basic.tl", String::from(include_str!("basic.tl")));

    // Parse
    let now = Instant::now();
    let mut lexer = lexer::Lexer::new(source.clone());
    let tokens = lexer.tokenize()?;
    let tokenization_time = now.elapsed();
    let now = Instant::now();
    let ast = ast::Parser::new(tokens, source.clone()).parse()?;
    let ast_gen_time = now.elapsed();

    // Evaluate
    let now = Instant::now();
    let evaluated = Scope::new(HashMap::new(), source, ast).eval()?;
    let eval_time = now.elapsed();

    println!(
        "Evaluated: {evaluated}\n\nTook {tokenization_time:?} to tokenize.\nTook {ast_gen_time:?} to generate an AST.\nTook {eval_time:?} to evaluate.\nTook {total_time:?} in total.",
        total_time = tokenization_time + ast_gen_time + eval_time
    );

    Ok(())
}
