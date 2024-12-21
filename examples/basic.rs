// Broken at the moment.

use logger::Log;
use std::{path::PathBuf, process, time::Instant};
use tl::{
    parser::{parse, tokenizer},
    Source,
};

fn main() {
    let source = Source::from(PathBuf::from("examples/basic.tl"));
    let now = Instant::now();
    let tokens = tokenizer::Parser::new(&source).tokenize().unwrap_or_else(|err| {
        Log::from(err).output();
        process::exit(0)
    });
    // let ast = parse(source).unwrap_or_else(|err| {
    //     Log::from(*err).output();
    //     process::exit(0)
    // });
    // Get elapsed time before pretty-printing the AST.
    let elapsed = now.elapsed();

    dbg!(tokens);
    println!("Took {elapsed:?}");
}
