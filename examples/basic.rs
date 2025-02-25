use logger::Log;
use std::{process, time::Instant};
use tl::runtime::Scope;
use tl::{Source, parser::parse};

fn main() {
    let source = Source::from_path("examples/basic.tl").expect("Failed to read source");

    // Parse
    let now = Instant::now();
    let ast = parse(&source).unwrap_or_else(|err| {
        Log::from(*err).output();
        process::exit(0)
    });
    let parse_time = now.elapsed();

    // Evaluate
    let now = Instant::now();
    let evaluated = Scope::new(source, ast).eval();
    let eval_time = now.elapsed();

    match evaluated {
        Ok(evaluated) => {
            println!("Evaluated: {evaluated}");
        }
        Err(err) => {
            Log::from(*err).output();
            return;
        }
    }

    println!(
        "Took {parse_time:?} to parse.\nTook {eval_time:?} to evaluate.\nTook {:?} in total.",
        parse_time + eval_time
    );
}
