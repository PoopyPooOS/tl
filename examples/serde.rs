use serde::Deserialize;
use std::time::Instant;
use tl::{Source, eval};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct User {
    name: String,
    age: u8,
}

fn main() {
    let source = Source::from_path("examples/serde.tl").expect("Failed to read source");
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
