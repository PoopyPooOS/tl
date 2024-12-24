// Broken at the moment.

use logger::Log;
use std::{path::PathBuf, process, time::Instant};
use tl::parser::ast;
#[allow(unused_imports)]
use tl::{
    parser::{parse, tokenizer},
    Source,
};

fn main() {
    let source = Source::from(PathBuf::from("examples/basic.tl"));
    let now = Instant::now();
    let tokens = tokenizer::Parser::new(source.clone()).tokenize().unwrap_or_else(|err| {
        Log::from(err).output();
        process::exit(0)
    });
    let ast = ast::Parser::new(tokens, source).parse().unwrap_or_else(|err| {
        Log::from(*err).output();
        process::exit(0)
    });
    let elapsed = now.elapsed();

    dbg!(&ast);
    println!("Took {elapsed:?}");
}
