#![feature(let_chains, new_range_api)]
#![allow(dead_code)]

use std::process;

use logger::Location;

#[cfg(test)]
mod tests;

mod ast;
// mod eval; TODO: Implement interpreter
mod tokenizer;
mod utils;

fn main() {
    let path = "main.tl";

    let tokens = match tokenizer::Parser::new(path).tokenize() {
        Ok(tokens) => tokens,
        Err(log) => {
            log.output();
            process::exit(1);
        }
    };

    for token in &tokens {
        println!(
            "{} from '{}' to '{}'",
            token.token_type,
            Location::new_with_section(path, token.line..=token.line, token.column..=token.column),
            Location::new_with_section(path, token.line..=token.line, token.column..=token.column + token.len)
        );
    }

    let mut ast = ast::Parser::new(tokens, path);
    let parsed_ast = match ast.parse() {
        Ok(parsed_ast) => parsed_ast,
        Err(log) => {
            log.output();
            process::exit(1);
        }
    };

    println!("{parsed_ast:#?}");
}
