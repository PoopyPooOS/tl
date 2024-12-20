use logger::Log;
use std::{path::PathBuf, process, time::Instant};
use tl::{parser::parse, Source};

fn main() {
    let source = Source::from(PathBuf::from("examples/basic.tl"));
    let now = Instant::now();
    let ast = parse(source).unwrap_or_else(|err| {
        Log::from(*err).output();
        process::exit(0)
    });
    // Get elapsed time before pretty-printing the AST.
    let elapsed = now.elapsed();

    dbg!(ast);
    println!("Took {elapsed:?}");
}
