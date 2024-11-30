use std::collections::HashMap;

use logger::Log;

use crate::{
    parser::{
        ast::types::{BinaryOperator, Expr, Literal, Statement},
        parse as full_parse,
    },
    source::Source,
};

fn parse(text: impl Into<String>) -> Result<Vec<Statement>, Box<Log>> {
    full_parse(Source::new(text))
}

macro_rules! literal {
    ($literal:ident($value:expr)) => {
        Statement::Expr(Expr::Literal(Literal::$literal($value)))
    };
}

macro_rules! box_literal {
    ($literal:ident($value:expr)) => {
        Box::new(Expr::Literal(Literal::$literal($value)))
    };
}

#[test]
fn boolean() {
    let input = "true";
    let expected = vec![literal!(Bool(true))];
    assert_eq!(parse(input).unwrap(), expected);

    let input = "false";
    let expected = vec![literal!(Bool(false))];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn number() {
    let input = "42";
    let expected = vec![literal!(Number(42))];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn float() {
    const PI: f64 = std::f64::consts::PI;

    let input = PI.to_string();
    let expected = vec![literal!(Float(PI))];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn string() {
    let input = "\"Hello, world!\"";
    let expected = vec![literal!(String("Hello, world!".to_string()))];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn escaped_string() {
    let input = "\"Hello, \\n\\tworld!\"";
    let expected = vec![literal!(String("Hello, \n\tworld!".to_string()))];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn binary_operators() {
    let input = "1 + 2 - 3 * 4 / 5";
    let expected = vec![Statement::Expr(Expr::BinaryOp {
        left: Box::new(Expr::BinaryOp {
            left: box_literal!(Number(1)),
            operator: BinaryOperator::Plus,
            right: box_literal!(Number(2)),
        }),
        operator: BinaryOperator::Minus,
        right: Box::new(Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: box_literal!(Number(3)),
                operator: BinaryOperator::Multiply,
                right: box_literal!(Number(4)),
            }),
            operator: BinaryOperator::Divide,
            right: box_literal!(Number(5)),
        }),
    })];

    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn object() {
    let input = "{ name: \"John Doe\" age = 42 }";
    let expected = vec![Statement::Expr(Expr::Literal(Literal::Object(HashMap::from([
        ("name".to_string(), Expr::Literal(Literal::String("John Doe".to_string()))),
        ("age".to_string(), Expr::Literal(Literal::Number(42))),
    ]))))];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn array() {
    let input = "[ 1 2 3 ]";
    let expected = vec![Statement::Expr(Expr::Literal(Literal::Array(vec![
        Expr::Literal(Literal::Number(1)),
        Expr::Literal(Literal::Number(2)),
        Expr::Literal(Literal::Number(3)),
    ])))];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn function_definition() {
    let input = r#"
        fn greet(name) {
            println("Hello, ${name}!")
        }

        greet("John Doe")
    "#;
    let expected = vec![
        Statement::Fn {
            name: "greet".to_string(),
            parameters: vec!["name".to_string()],
            body: vec![Statement::Expr(Expr::Call {
                name: "println".to_string(),
                args: vec![Expr::Literal(Literal::InterpolatedString(vec![
                    Expr::Literal(Literal::String("Hello, ".to_string())),
                    Expr::Identifier("name".to_string()),
                    Expr::Literal(Literal::String("!".to_string())),
                ]))],
            })],
        },
        Statement::Expr(Expr::Call {
            name: "greet".to_string(),
            args: vec![Expr::Literal(Literal::String("John Doe".to_string()))],
        }),
    ];
    assert_eq!(parse(input).unwrap(), expected);
}

// TODO: Add support for this case.
#[ignore = "not implemented yet"]
#[test]
fn round_brackets() {
    let input = "(2 + 3) * 4";
    let expected = vec![Statement::Expr(Expr::BinaryOp {
        left: Box::new(Expr::BinaryOp {
            left: box_literal!(Number(2)),
            operator: BinaryOperator::Plus,
            right: box_literal!(Number(3)),
        }),
        operator: BinaryOperator::Multiply,
        right: box_literal!(Number(4)),
    })];
    assert_eq!(parse(input).unwrap(), expected);
}
