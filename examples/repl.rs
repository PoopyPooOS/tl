use std::{env, io::Write, process::Command, time::Instant};
use tl::Source;

fn main() {
    println!(
        "This is a very basic REPL. Its highly recommended to use your systems default editor with the `.e` command.\n`CTRL-C` to quit."
    );
    loop {
        let mut input = input("> ");

        if input == ".e" {
            let Ok(editor) = env::var("EDITOR") else {
                println!("EDITOR environment variable is not set");
                continue;
            };

            Command::new(editor)
                .arg("/tmp/repl.tl")
                .spawn()
                .expect("Failed to open editor")
                .wait()
                .expect("Failed to open editor");

            input = std::fs::read_to_string("/tmp/repl.tl").expect("Failed to read '/tmp/repl.tl'");
        }

        let source = Source::new(input);
        let now = Instant::now();
        match tl::eval_untyped(source) {
            Ok(value) => {
                println!("Took {:?} to evaluate:", now.elapsed());

                if let Some(value) = value {
                    println!("{value:#?}");
                } else {
                    println!("Evaluated to nothing");
                }
            }
            Err(log) => {
                log.output();
                break;
            }
        };
    }
}

fn input(prefix: impl Into<String>) -> String {
    print!("{}", prefix.into());
    std::io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read from stdin");

    input.trim().to_string()
}
