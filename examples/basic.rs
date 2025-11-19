use miette::NamedSource;
use std::{path::PathBuf, time::Instant};
use tl::{
    Source, eval_untyped, object,
    runtime::{Scope, Value},
};

fn main() -> miette::Result<()> {
    let source = Source::new(
        NamedSource::new("examples/basic.tl", String::from(include_str!("basic.tl"))),
        Some(PathBuf::from("examples/basic.tl")),
    );

    let now = Instant::now();

    let scope_setup = |scope: &mut Scope| {
        scope.define(
            "system",
            object! {
                arch: "aarch64",
                os: "android",
                brand: "nothing",
                model: "spacewar",
            },
        );
    };

    let value = eval_untyped(source, scope_setup)?;

    let time = now.elapsed();
    println!("Evaluated:\n{value}\nTook {time:?}.");

    Ok(())
}
