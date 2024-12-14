use std::path::PathBuf;
use tl::{eval, Source};

#[derive(Debug, serde::Deserialize)]
enum UserType {
    Admin,
    User,
    Guest,
}

fn main() {
    let source = Source::from(PathBuf::from("examples/enum.tl"));
    println!("Evaluating:\n```\n{source}\n```");

    match eval::<UserType>(source) {
        Ok(Some(v)) => match v {
            UserType::Admin => println!("You are an Admin"),
            UserType::User => println!("You are a normal User"),
            UserType::Guest => println!("You are a Guest"),
        },
        Ok(None) => (),
        Err(log) => log.output(),
    }
}
