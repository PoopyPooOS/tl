#![feature(let_chains, new_range_api)]
#![allow(dead_code)]

#[cfg(test)]
mod tests;

// mod eval; TODO: Implement interpreter
pub mod parser;
pub mod runtime;
pub mod source;
mod utils;

#[cfg(feature = "serde")]
pub use crate::utils::eval_untyped;
pub use crate::{parser::parse, source::Source, utils::eval};

/// Macro for defining examples
#[macro_export]
macro_rules! example {
    ($name:ident) => {
        use std::{path::PathBuf, process, time::Instant};
        use tl::{parser::parse, source::Source};

        fn main() {
            let path = PathBuf::from(concat!("examples/", stringify!($name), ".tl"));
            let source = Source::from(path);

            println!("Parsing:\n```\n{}\n```\n", source.text);
            let now = Instant::now();
            match parse(source) {
                Ok(ast) => {
                    println!("Took {:?} to generate AST:", now.elapsed());
                    println!("```\n{ast:#?}\n```");
                }
                Err(log) => {
                    log.output();
                    process::exit(1);
                }
            };
        }
    };
}
