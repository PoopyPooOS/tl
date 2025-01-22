use serde::Deserialize;
use std::{path::PathBuf, time::Instant};
use tl::{utils::eval, Source};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct User {
    name: String,
    age: u8,
}

fn main() {
    let source = Source::from(PathBuf::from("examples/serde.tl"));
    let now = Instant::now();

    match eval::<Vec<User>>(source) {
        Ok(Some(value)) => {
            let time = now.elapsed();
            println!("Deserialized: {value:#?}\nTook {time:?}.");
        }
        Ok(None) => (),
        Err(log) => log.output(),
    }
}
