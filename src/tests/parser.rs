#![allow(clippy::unwrap_used, reason = "Panics automatically invalidate tests")]
#![allow(
    clippy::reversed_empty_ranges,
    reason = "Some section ranges may not make sense at first but they are proper ranges"
)]

use crate::{
    parser::{
        ast::types::{BinaryOperator, Expr, ExprType, Literal, Statement, StatementType},
        parse as full_parse,
    },
    source::Source,
};
use logger::{location::Section, Log};
use pretty_assertions::assert_eq;
use std::{collections::BTreeMap, path::PathBuf};

fn parse(text: impl Into<String>) -> Result<Vec<Statement>, Box<Log>> {
    Ok(full_parse(&Source::new(text)).map_err(|err| Log::from(*err))?)
}

macro_rules! literal {
    ($literal:ident($value:expr), $section:expr) => {
        Statement::new(
            StatementType::Expr(Expr::new(
                ExprType::Literal(Literal::$literal($value)),
                $section,
            )),
            $section,
        )
    };
}

macro_rules! box_literal {
    ($literal:ident($value:expr), $section:expr) => {
        Box::new(Expr::new(
            ExprType::Literal(Literal::$literal($value)),
            $section,
        ))
    };
}

#[test]
fn boolean() {
    let input = "true";
    let expected = vec![literal!(Boolean(true), Section::new(0..=0, 0..=4))];
    assert_eq!(parse(input).unwrap(), expected);

    let input = "false";
    let expected = vec![literal!(Boolean(false), Section::new(0..=0, 0..=5))];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn number() {
    let input = "42";
    let expected = vec![literal!(Int(42), Section::new(0..=0, 0..=2))];
    assert_eq!(parse(input).unwrap(), expected);
}

#[allow(clippy::approx_constant)]
#[test]
fn float() {
    let input = "3.14";
    let expected = vec![literal!(Float(3.14), Section::new(0..=0, 0..=4))];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn string() {
    let input = "\"Hello, world!\"";
    let expected = vec![literal!(
        String("Hello, world!".to_string()),
        Section::new(0..=0, 0..=15)
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn escaped_string() {
    let input = "\"Hello, \\n\\tworld!\"";
    let expected = vec![literal!(
        String("Hello, \n\tworld!".to_string()),
        Section::new(0..=0, 0..=19)
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn interpolated_string() {
    let input = "\"Hello, my name is ${name}!\"";
    let expected = vec![literal!(
        InterpolatedString(vec![
            Expr::new(
                ExprType::Literal(Literal::String("Hello, my name is ".to_string())),
                Section::new(0..=0, 1..=19)
            ),
            Expr::new(
                ExprType::Identifier("name".to_string()),
                Section::new(0..=0, 21..=25)
            ),
            Expr::new(
                ExprType::Literal(Literal::String("!".to_string())),
                Section::new(0..=0, 26..=27)
            ),
        ]),
        Section::new(0..=0, 0..=28)
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn relative_path() {
    let input = "./file.txt";
    let expected = vec![literal!(
        Path(PathBuf::from("file.txt")),
        Section::new(0..=0, 0..=10)
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn absolute_path() {
    let input = "/bin/sh";
    let expected = vec![literal!(
        Path(PathBuf::from("/bin/sh")),
        Section::new(0..=0, 0..=7)
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
                Expr::new(ExprType::Literal(Literal::String("John Doe".to_string())), Section::new(0..=0, 9..=19))
            ),
            (
                "age".to_string(),
                Expr::new(ExprType::Literal(Literal::Int(42)), Section::new(0..=0, 26..=28))
            ),
        ])
        ),
        Section::new(0..=0, 0..=30)
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn field_access() {
    let input = "package.dependencies";
    let expected = vec![Statement::new(
        StatementType::Expr(Expr::new(
            ExprType::FieldAccess {
                base: Box::new(Expr::new(
                    ExprType::Identifier("package".into()),
                    Section::new(0..=0, 0..=7),
                )),
                path: vec![Expr::new(
                    ExprType::Identifier("dependencies".into()),
                    Section::new(0..=0, 8..=20),
                )],
            },
            Section::new(0..=0, 0..=20),
        )),
        Section::new(0..=0, 0..=20),
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn array() {
    let input = "[ 1 2 3 ]";
    let expected = vec![literal!(
        Array(vec![
            Expr::new(
                ExprType::Literal(Literal::Int(1)),
                Section::new(0..=0, 2..=3)
            ),
            Expr::new(
                ExprType::Literal(Literal::Int(2)),
                Section::new(0..=0, 4..=5)
            ),
            Expr::new(
                ExprType::Literal(Literal::Int(3)),
                Section::new(0..=0, 6..=7)
            ),
        ]),
        Section::new(0..=0, 0..=9)
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
                Section::new(0..=0, 1..=5),
            ))),
            Section::new(0..=0, 0..=5),
        )),
        Section::new(0..=0, 0..=5),
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
                                    Section::new(1..=1, 12..=20),
                                )],
                            },
                            Section::new(1..=1, 4..=21),
                        )),
                        Section::new(1..=1, 4..=21),
                    )],
                },
                Section::new(0..=2, 15..=1),
            ),
        },
        Section::new(0..=2, 0..=1),
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
                                Section::new(1..=1, 5..=12)
                            ),
                            Expr::new(
                                ExprType::Identifier("name".into()),
                                Section::new(1..=1, 14..=18)
                            ),
                            Expr::new(
                                ExprType::Literal(Literal::String("!".into())),
                                Section::new(1..=1, 19..=20)
                            ),
                        ]),
                        Section::new(1..=1, 4..=21)
                    )],
                },
                Section::new(0..=2, 12..=1),
            ),
        },
        Section::new(0..=2, 0..=1),
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
                                Section::new(1..=1, 5..=12)
                            ),
                            Expr::new(
                                ExprType::Identifier("name".into()),
                                Section::new(1..=1, 14..=18)
                            ),
                            Expr::new(
                                ExprType::Literal(Literal::String("! You are ".into())),
                                Section::new(1..=1, 20..=29)
                            ),
                            Expr::new(
                                ExprType::Identifier("age".into()),
                                Section::new(1..=1, 31..=34)
                            ),
                            Expr::new(
                                ExprType::Literal(Literal::String(" years old.".into())),
                                Section::new(1..=1, 35..=46)
                            ),
                        ]),
                        Section::new(1..=1, 4..=47)
                    )],
                },
                Section::new(0..=2, 15..=1),
            ),
        },
        Section::new(0..=2, 0..=1),
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
                                                    Section::new(2..=2, 8..=16),
                                                )),
                                                operator: BinaryOperator::Eq,
                                                right: Box::new(Expr::new(
                                                    ExprType::Literal(Literal::Int(0)),
                                                    Section::new(2..=2, 20..=21),
                                                )),
                                            },
                                            Section::new(2..=2, 8..=21),
                                        ),
                                        Expr::new(
                                            ExprType::Literal(Literal::Int(1)),
                                            Section::new(3..=3, 8..=9),
                                        ),
                                        Expr::new(
                                            ExprType::BinaryOp {
                                                left: Box::new(Expr::new(
                                                    ExprType::Identifier("base".into()),
                                                    Section::new(4..=4, 8..=12),
                                                )),
                                                operator: BinaryOperator::Multiply,
                                                right: Box::new(Expr::new(
                                                    ExprType::Call {
                                                        name: "pow".into(),
                                                        args: vec![
                                                            Expr::new(
                                                                ExprType::Identifier("base".into()),
                                                                Section::new(4..=4, 19..=23),
                                                            ),
                                                            Expr::new(
                                                                ExprType::BinaryOp {
                                                                    left: Box::new(Expr::new(
                                                                        ExprType::Identifier(
                                                                            "exponent".into(),
                                                                        ),
                                                                        Section::new(
                                                                            4..=4,
                                                                            24..=32,
                                                                        ),
                                                                    )),
                                                                    operator: BinaryOperator::Minus,
                                                                    right: Box::new(Expr::new(
                                                                        ExprType::Literal(
                                                                            Literal::Int(1),
                                                                        ),
                                                                        Section::new(
                                                                            4..=4,
                                                                            35..=36,
                                                                        ),
                                                                    )),
                                                                },
                                                                Section::new(4..=4, 24..=36),
                                                            ),
                                                        ],
                                                    },
                                                    Section::new(4..=4, 15..=37),
                                                )),
                                            },
                                            Section::new(4..=4, 8..=37),
                                        ),
                                    ],
                                },
                                Section::new(1..=5, 4..=5),
                            )),
                            Section::new(1..=5, 4..=5),
                        )],
                    },
                    Section::new(0..=6, 10..=1),
                ),
            },
            Section::new(0..=6, 0..=1),
        ),
        Statement::new(
            StatementType::Expr(Expr::new(
                ExprType::Call {
                    name: "pow".into(),
                    args: vec![
                        Expr::new(
                            ExprType::Literal(Literal::Int(2)),
                            Section::new(8..=8, 4..=5),
                        ),
                        Expr::new(
                            ExprType::Literal(Literal::Int(10)),
                            Section::new(8..=8, 6..=8),
                        ),
                    ],
                },
                Section::new(8..=8, 0..=9),
            )),
            Section::new(8..=8, 0..=9),
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
            Section::new(0..=0, 0..=6),
        )),
        Section::new(0..=0, 0..=6),
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
                    Section::new(0..=0, 8..=23),
                )],
            },
            Section::new(0..=0, 0..=24),
        )),
        Section::new(0..=0, 0..=24),
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
                                Section::new(1..=1, 4..=12),
                            )),
                            operator: BinaryOperator::Eq,
                            right: Box::new(Expr::new(
                                ExprType::Literal(Literal::String(
                                    "strongpassoword123".to_string(),
                                )),
                                Section::new(1..=1, 16..=36),
                            )),
                        },
                        Section::new(1..=1, 4..=36),
                    ),
                    Expr::new(
                        ExprType::Literal(Literal::String("Password is correct".to_string())),
                        Section::new(2..=2, 4..=25),
                    ),
                    Expr::new(
                        ExprType::Literal(Literal::String("Password is incorrect".to_string())),
                        Section::new(3..=3, 4..=27),
                    ),
                ],
            },
            Section::new(0..=4, 0..=1),
        )),
        Section::new(0..=4, 0..=1),
    )];
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn binary_op() {
    let input = "2 + 3 * 4";
    let expected = vec![Statement::new(
        StatementType::Expr(Expr::new(
            ExprType::BinaryOp {
                left: box_literal!(Int(2), Section::new(0..=0, 0..=1)),
                operator: BinaryOperator::Plus,
                right: Box::new(Expr::new(
                    ExprType::BinaryOp {
                        left: box_literal!(Int(3), Section::new(0..=0, 4..=5)),
                        operator: BinaryOperator::Multiply,
                        right: box_literal!(Int(4), Section::new(0..=0, 8..=9)),
                    },
                    Section::new(0..=0, 4..=9),
                )),
            },
            Section::new(0..=0, 0..=9),
        )),
        Section::new(0..=0, 0..=9),
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
                Section::new(0..=0, 11..=21),
            ),
        },
        Section::new(0..=0, 0..=21),
    )];
    assert_eq!(parse(input).unwrap(), expected);
}
