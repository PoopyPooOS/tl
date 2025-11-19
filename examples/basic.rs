use miette::NamedSource;
use std::{path::PathBuf, time::Instant};
use tl::{Source, eval_untyped};

fn main() -> miette::Result<()> {
    let source = Source::new(
        NamedSource::new("examples/basic.tl", String::from(include_str!("basic.tl"))),
        Some(PathBuf::from("examples/basic.tl")),
    );

    let now = Instant::now();

    let value = eval_untyped(source, |_| ())?;

    let time = now.elapsed();
    println!("Evaluated:\n{value}\nTook {time:?}.");

    Ok(())
}
