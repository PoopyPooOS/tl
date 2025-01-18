#![allow(clippy::unwrap_used, reason = "Panics automatically invalidate tests")]
#![allow(clippy::too_many_lines, reason = "Some tests can get very long")]

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
    Ok(full_parse(&Source::new(text)).map_err(|err| Log::from(*err))?)
}

macro_rules! literal {
    ($literal:ident($value:expr), $line:expr, $cols:expr) => {
        Statement::new(
            StatementType::Expr(Expr::new(
                ExprType::Literal(Literal::$literal($value)),
                $line,
                $cols,
            )),
            $line,
            $cols,
        )
    };
}

#[allow(unused_macros, reason = "Will be used")]
macro_rules! box_literal {
    ($literal:ident($value:expr), $line:expr, $cols:expr) => {
        Box::new(Expr::new(
            ExprType::Literal(Literal::$literal($value)),
            $line,
            $cols,
        ))
    };
}

#[test]
fn boolean() {
    let input = "true";
    let expected = vec![literal!(Boolean(true), 0, 0..=4)];
    assert_eq!(parse(input).unwrap(), expected);

    let input = "false";
    let expected = vec![literal!(Boolean(false), 0, 0..=5)];
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
            Expr::new(
                ExprType::Literal(Literal::String("Hello, my name is ".to_string())),
                0,
                1..=19
            ),
            Expr::new(ExprType::Identifier("name".to_string()), 0, 21..=25),
            Expr::new(
                ExprType::Literal(Literal::String("!".to_string())),
                0,
                26..=27
            ),
        ]),
        0,
        0..=28
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn object() {
    let input = "{ name = \"John Doe\" age = 42 }";
    let expected = vec![literal!(
        Object(
            #[rustfmt::skip]
            BTreeMap::from([
            (
                "name".to_string(),
                Expr::new(ExprType::Literal(Literal::String("John Doe".to_string())), 0, 9..=19)
            ),
            (
                "age".to_string(),
                Expr::new(ExprType::Literal(Literal::Int(42)), 0, 26..=28)
            ),
        ])
        ),
        0,
        0..=30
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn field_access() {
    let input = "package.dependencies";
    let expected = vec![Statement::new(
        StatementType::Expr(Expr::new(
            ExprType::BinaryOp {
                left: Box::new(Expr::new(ExprType::Identifier("package".into()), 0, 0..=7)),
                operator: BinaryOperator::Dot,
                right: Box::new(Expr::new(
                    ExprType::Identifier("dependencies".into()),
                    0,
                    8..=20,
                )),
            },
            0,
            0..=20,
        )),
        0,
        0..=20,
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
            ExprType::Not(Box::new(Expr::new(
                ExprType::Literal(Literal::Boolean(true)),
                0,
                1..=5,
            ))),
            0,
            0..=5,
        )),
        0,
        0..=5,
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn function_declaration() {
    // No arguments
    #[rustfmt::skip]
    let input = r#"let do_thing = () {
    println("Hello!")
}"#;
    let expected = vec![Statement::new(
        StatementType::Let {
            name: "do_thing".into(),
            value: Expr::new(
                ExprType::FnDecl {
                    args: vec![],
                    body: vec![Statement::new(
                        StatementType::Expr(Expr::new(
                            ExprType::Call {
                                name: "println".into(),
                                args: vec![Expr::new(
                                    ExprType::Literal(Literal::String("Hello!".into())),
                                    1,
                                    12..=20,
                                )],
                            },
                            1,
                            4..=21,
                        )),
                        1,
                        4..=21,
                    )],
                },
                0,
                15..=19,
            ),
        },
        0,
        0..=19,
    )];
    assert_eq!(parse(input).unwrap(), expected);

    // Single argument
    #[rustfmt::skip]
    let input = r#"let greet = (name) {
    "Hello, ${name}!"
}"#;
    let expected = vec![Statement::new(
        StatementType::Let {
            name: "greet".into(),
            value: Expr::new(
                ExprType::FnDecl {
                    args: vec!["name".into()],
                    body: vec![literal!(
                        InterpolatedString(vec![
                            Expr::new(
                                ExprType::Literal(Literal::String("Hello, ".into())),
                                1,
                                5..=12
                            ),
                            Expr::new(ExprType::Identifier("name".into()), 1, 14..=18),
                            Expr::new(ExprType::Literal(Literal::String("!".into())), 1, 19..=20),
                        ]),
                        1,
                        4..=21
                    )],
                },
                0,
                12..=20,
            ),
        },
        0,
        0..=20,
    )];
    assert_eq!(parse(input).unwrap(), expected);

    // Multiple arguments
    #[rustfmt::skip]
    let input = r#"let do_thing = (name age) {
    "Hello, ${name}! You are ${age} years old."
}"#;
    let expected = vec![Statement::new(
        StatementType::Let {
            name: "do_thing".into(),
            value: Expr::new(
                ExprType::FnDecl {
                    args: vec!["name".into(), "age".into()],
                    body: vec![literal!(
                        InterpolatedString(vec![
                            Expr::new(
                                ExprType::Literal(Literal::String("Hello, ".into())),
                                1,
                                5..=12
                            ),
                            Expr::new(ExprType::Identifier("name".into()), 1, 14..=18),
                            Expr::new(
                                ExprType::Literal(Literal::String("! You are ".into())),
                                1,
                                20..=29
                            ),
                            Expr::new(ExprType::Identifier("age".into()), 1, 31..=34),
                            Expr::new(
                                ExprType::Literal(Literal::String(" years old.".into())),
                                1,
                                35..=46
                            ),
                        ]),
                        1,
                        4..=47
                    )],
                },
                0,
                15..=27,
            ),
        },
        0,
        0..=27,
    )];
    assert_eq!(parse(input).unwrap(), expected);

    // Complex
    #[rustfmt::skip]
    let input = r"let pow = (base exponent) {
    if(
        exponent == 0
        1
        base * pow(base exponent - 1)
    )
}

pow(2 10)";
    let expected = vec![
        Statement::new(
            StatementType::Let {
                name: "pow".into(),
                value: Expr::new(
                    ExprType::FnDecl {
                        args: vec!["base".into(), "exponent".into()],
                        body: vec![Statement::new(
                            StatementType::Expr(Expr::new(
                                ExprType::Call {
                                    name: "if".into(),
                                    args: vec![
                                        Expr::new(
                                            ExprType::BinaryOp {
                                                left: Box::new(Expr::new(
                                                    ExprType::Identifier("exponent".into()),
                                                    2,
                                                    8..=16,
                                                )),
                                                operator: BinaryOperator::Eq,
                                                right: Box::new(Expr::new(
                                                    ExprType::Literal(Literal::Int(0)),
                                                    2,
                                                    20..=21,
                                                )),
                                            },
                                            2,
                                            8..=21,
                                        ),
                                        Expr::new(ExprType::Literal(Literal::Int(1)), 3, 8..=9),
                                        Expr::new(
                                            ExprType::BinaryOp {
                                                left: Box::new(Expr::new(
                                                    ExprType::Identifier("base".into()),
                                                    4,
                                                    8..=12,
                                                )),
                                                operator: BinaryOperator::Multiply,
                                                right: Box::new(Expr::new(
                                                    ExprType::Call {
                                                        name: "pow".into(),
                                                        args: vec![
                                                            Expr::new(
                                                                ExprType::Identifier("base".into()),
                                                                4,
                                                                19..=23,
                                                            ),
                                                            Expr::new(
                                                                ExprType::BinaryOp {
                                                                    left: Box::new(Expr::new(
                                                                        ExprType::Identifier(
                                                                            "exponent".into(),
                                                                        ),
                                                                        4,
                                                                        24..=32,
                                                                    )),
                                                                    operator: BinaryOperator::Minus,
                                                                    right: Box::new(Expr::new(
                                                                        ExprType::Literal(
                                                                            Literal::Int(1),
                                                                        ),
                                                                        4,
                                                                        35..=36,
                                                                    )),
                                                                },
                                                                4,
                                                                24..=36,
                                                            ),
                                                        ],
                                                    },
                                                    4,
                                                    15..=37,
                                                )),
                                            },
                                            4,
                                            8..=37,
                                        ),
                                    ],
                                },
                                1,
                                4..=7,
                            )),
                            1,
                            4..=7,
                        )],
                    },
                    0,
                    10..=27,
                ),
            },
            0,
            0..=27,
        ),
        Statement::new(
            StatementType::Expr(Expr::new(
                ExprType::Call {
                    name: "pow".into(),
                    args: vec![
                        Expr::new(ExprType::Literal(Literal::Int(2)), 8, 4..=5),
                        Expr::new(ExprType::Literal(Literal::Int(10)), 8, 6..=8),
                    ],
                },
                8,
                0..=9,
            )),
            8,
            0..=9,
        ),
    ];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn call() {
    // No arguments
    let input = "exit()";
    let expected = vec![Statement::new(
        StatementType::Expr(Expr::new(
            ExprType::Call {
                name: "exit".to_string(),
                args: vec![],
            },
            0,
            0..=6,
        )),
        0,
        0..=6,
    )];
    assert_eq!(parse(input).unwrap(), expected);

    // Single argument
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

    // Multiple arguments
    let input = r#"if(
    password == "strongpassoword123"
    "Password is correct"
    "Password is incorrect"
)"#;
    let expected = vec![Statement::new(
        StatementType::Expr(Expr::new(
            ExprType::Call {
                name: "if".to_string(),
                args: vec![
                    Expr::new(
                        ExprType::BinaryOp {
                            left: Box::new(Expr::new(
                                ExprType::Identifier("password".to_string()),
                                1,
                                4..=12,
                            )),
                            operator: BinaryOperator::Eq,
                            right: Box::new(Expr::new(
                                ExprType::Literal(Literal::String(
                                    "strongpassoword123".to_string(),
                                )),
                                1,
                                16..=36,
                            )),
                        },
                        1,
                        4..=36,
                    ),
                    Expr::new(
                        ExprType::Literal(Literal::String("Password is correct".to_string())),
                        2,
                        4..=25,
                    ),
                    Expr::new(
                        ExprType::Literal(Literal::String("Password is incorrect".to_string())),
                        3,
                        4..=27,
                    ),
                ],
            },
            0,
            0..=3,
        )),
        0,
        0..=3,
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn binary_op() {
    let input = "2 + 3 * 4";
    let expected = vec![Statement::new(
        StatementType::Expr(Expr::new(
            ExprType::BinaryOp {
                left: box_literal!(Int(2), 0, 0..=1),
                operator: BinaryOperator::Plus,
                right: Box::new(Expr::new(
                    ExprType::BinaryOp {
                        left: box_literal!(Int(3), 0, 4..=5),
                        operator: BinaryOperator::Multiply,
                        right: box_literal!(Int(4), 0, 8..=9),
                    },
                    0,
                    4..=9,
                )),
            },
            0,
            0..=9,
        )),
        0,
        0..=9,
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn variable() {
    let input = "let name = \"John Doe\"";
    let expected = vec![Statement::new(
        StatementType::Let {
            name: "name".to_string(),
            value: Expr::new(
                ExprType::Literal(Literal::String("John Doe".to_string())),
                0,
                11..=21,
            ),
        },
        0,
        0..=21,
    )];
    assert_eq!(parse(input).unwrap(), expected);
}
