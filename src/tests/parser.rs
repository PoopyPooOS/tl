#![allow(clippy::unwrap_used, reason = "Panics automatically invalidate tests")]

use crate::{
    parser::{
        ast::types::{BinaryOperator, Expr, ExprType, Literal, Statement, StatementType},
        parse as full_parse,
    },
    source::Source,
};
use logger::Log;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

fn parse(text: impl Into<String>) -> Result<Vec<Statement>, Box<Log>> {
    Ok(full_parse(Source::new(text)).map_err(|err| Log::from(*err))?)
}

macro_rules! literal {
    ($literal:ident($value:expr), $line:expr, $cols:expr) => {
        Statement::new(
            StatementType::Expr(Expr::new(ExprType::Literal(Literal::$literal($value)), $line, $cols)),
            $line,
            $cols,
        )
    };
}

#[allow(unused_macros, reason = "Will be used")]
macro_rules! box_literal {
    ($literal:ident($value:expr), $line:expr, $cols:expr) => {
        Box::new(Expr::new(ExprType::Literal(Literal::$literal($value)), $line, $cols))
    };
}

#[test]
fn boolean() {
    let input = "true";
    let expected = vec![literal!(Bool(true), 0, 0..=4)];
    assert_eq!(parse(input).unwrap(), expected);

    let input = "false";
    let expected = vec![literal!(Bool(false), 0, 0..=5)];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn number() {
    let input = "42";
    let expected = vec![literal!(Int(42), 0, 0..=2)];
    assert_eq!(parse(input).unwrap(), expected);
}

#[allow(clippy::approx_constant)]
#[test]
fn float() {
    let input = "3.14";
    let expected = vec![literal!(Float(3.14), 0, 0..=4)];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn string() {
    let input = "\"Hello, world!\"";
    let expected = vec![literal!(String("Hello, world!".to_string()), 0, 0..=15)];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn escaped_string() {
    let input = "\"Hello, \\n\\tworld!\"";
    let expected = vec![literal!(String("Hello, \n\tworld!".to_string()), 0, 0..=19)];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn interpolated_string() {
    let input = "\"Hello, my name is ${name}!\"";
    let expected = vec![literal!(
        InterpolatedString(vec![
            Expr::new(ExprType::Literal(Literal::String("Hello, my name is ".to_string())), 0, 0..=19),
            Expr::new(ExprType::Identifier("name".to_string()), 0, 21..=25),
            Expr::new(ExprType::Literal(Literal::String("!".to_string())), 0, 0..=28),
        ]),
        0,
        0..=28
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn object() {
    let input = "{ name: \"John Doe\" age = 42 }";
    let expected = vec![literal!(
        Object(
            #[rustfmt::skip]
            BTreeMap::from([
            (
                "name".to_string(),
                Expr::new(ExprType::Literal(Literal::String("John Doe".to_string())), 0, 8..=18)
            ),
            (
                "age".to_string(), 
                Expr::new(ExprType::Literal(Literal::Int(42)), 0, 25..=27)
            ),
        ])
        ),
        0,
        0..=29
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn array() {
    let input = "[ 1 2 3 ]";
    let expected = vec![literal!(
        Array(vec![
            Expr::new(ExprType::Literal(Literal::Int(1)), 0, 2..=3),
            Expr::new(ExprType::Literal(Literal::Int(2)), 0, 4..=5),
            Expr::new(ExprType::Literal(Literal::Int(3)), 0, 6..=7),
        ]),
        0,
        0..=9
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn not() {
    let input = "!true";
    let expected = vec![Statement::new(
        StatementType::Expr(Expr::new(
            ExprType::Not(Box::new(Expr::new(ExprType::Literal(Literal::Bool(true)), 0, 1..=5))),
            0,
            0..=5,
        )),
        0,
        0..=5,
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn call() {
    let input = "println(\"Hello, world!\")";
    let expected = vec![Statement::new(
        StatementType::Expr(Expr::new(
            ExprType::Call {
                name: "println".to_string(),
                args: vec![Expr::new(
                    ExprType::Literal(Literal::String("Hello, world!".to_string())),
                    0,
                    8..=23,
                )],
            },
            0,
            0..=24,
        )),
        0,
        0..=24,
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn binary_op() {
    let input = "(2 + 3) * 4";
    let expected = vec![Statement::new(
        StatementType::Expr(Expr::new(
            ExprType::BinaryOp {
                left: Box::new(Expr::new(
                    ExprType::BinaryOp {
                        left: box_literal!(Int(2), 0, 1..=2),
                        operator: BinaryOperator::Plus,
                        right: box_literal!(Int(3), 0, 5..=6),
                    },
                    0,
                    1..=6,
                )),
                operator: BinaryOperator::Multiply,
                right: box_literal!(Int(4), 0, 10..=11),
            },
            0,
            0..=11,
        )),
        0,
        0..=11,
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn variable() {
    let input = "let name = \"John Doe\"";
    let expected = vec![Statement::new(
        StatementType::Let {
            name: "name".to_string(),
            value: Expr::new(ExprType::Literal(Literal::String("John Doe".to_string())), 0, 11..=21),
        },
        0,
        0..=21,
    )];
    assert_eq!(parse(input).unwrap(), expected);
}
