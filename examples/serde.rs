use std::path::PathBuf;
use tl::{eval, Source};

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct User {
    name: String,
    age: u8,
}

fn main() {
    let source = Source::from(PathBuf::from("examples/serde.tl"));
    println!("Evaluating:\n```\n{source}\n```");

    match eval::<Vec<User>>(source) {
        Ok(Some(value)) => println!("Deserialized: {value:#?}"),
        Ok(None) => (),
        Err(log) => log.output(),
    }
}
