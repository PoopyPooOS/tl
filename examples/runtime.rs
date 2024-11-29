use logger::info;
use std::{path::PathBuf, process, time::Instant};
use tl::{parser::parse, runtime::Scope, source::Source};

fn main() {
    let path = PathBuf::from("examples/runtime.tl");
    let source = Source::from(path);

    println!("Evaluating:\n```\n{}\n```", source.text);
    let ast_now = Instant::now();
    let ast_duration;

    let ast = match parse(source) {
        Ok(ast) => {
            ast_duration = ast_now.elapsed();
            ast
        }
        Err(log) => {
            log.output();
            process::exit(1);
        }
    };

    let eval_now = Instant::now();
    let eval_duration;

    let mut runtime = Scope::new(ast);
    match runtime.eval() {
        Ok(Some(value)) => {
            eval_duration = eval_now.elapsed();
            info!(format!("Evaluated: {value}\n"));
        }
        Ok(None) => {
            eval_duration = eval_now.elapsed();
            info!("No value evaluated.\n");
        }
        Err(log) => {
            log.output();
            process::exit(1);
        }
    }

    info!(format!("Took {ast_duration:?} to parse AST."));
    info!(format!("Took {eval_duration:?} to evaluate AST."));
    info!(format!("Took {:?} in total.", ast_duration + eval_duration));
}
