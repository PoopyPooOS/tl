#![feature(let_chains)]

use std::fs;

#[cfg(test)]
mod tests;

// mod ast; TODO: Implement AST
// mod eval; TODO: Implement interpreter
mod tokenizer;
mod utils;

fn main() {
    let input = fs::read_to_string("main.tl").expect("Failed to read file");
    let tokens = tokenizer::tokenize(input).expect("Failed to tokenize input");
    println!("Tokenizer output:\n{tokens:#?}");
}
