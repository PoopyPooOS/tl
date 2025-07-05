use std::time::Instant;
use tl::{eval, Source};

fn main() {
    let source = Source::from_text(include_str!("basic.tl"));
    let now = Instant::now();

    match eval::<String>(source) {
        Ok(Some(value)) => {
            let time = now.elapsed();
            println!("Evaluated:\n{value}\nTook {time:?}.");
        }
        Ok(None) => (),
        Err(log) => log.output(),
    }
}
