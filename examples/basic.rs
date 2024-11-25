use std::{path::PathBuf, process, time::Instant};
use tl::{parser::parse, source::Source};

fn main() {
    let path = PathBuf::from("examples/example.tl");
    let source = Source::from(path);

    println!("Parsing:\n```\n{}\n```\n", source.text);
    let now = Instant::now();
    let ast = match parse(source) {
        Ok(ast) => {
            println!("Took {:?} to generate AST.", now.elapsed());
            ast
        }
        Err(log) => {
            log.output();
            process::exit(1);
        }
    };

    println!("AST:\n```\n{ast:#?}\n```");
}
