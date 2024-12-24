// Broken at the moment.

use logger::Log;
use std::{path::PathBuf, process, time::Instant};
#[allow(unused_imports)]
use tl::{
    parser::{parse, tokenizer},
    Source,
};

fn main() {
    let source = Source::from(PathBuf::from("examples/basic.tl"));
    let now = Instant::now();
    let ast = parse(source).unwrap_or_else(|err| {
        Log::from(*err).output();
        process::exit(0)
    });
    let elapsed = now.elapsed();

    dbg!(&ast);
    println!("Took {elapsed:?}");
}
