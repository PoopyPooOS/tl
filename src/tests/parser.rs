#![allow(clippy::unwrap_used, reason = "Panics automatically invalidate tests")]

use crate::{
    parser::{
        self,
        ast::types::{BinaryOperator, Expr, ExprKind, Literal},
    },
    span,
};
use miette::NamedSource;
use pretty_assertions::assert_eq;
use std::{collections::BTreeMap, path::PathBuf};

fn parse(text: impl Into<String>) -> miette::Result<Expr> {
    Ok(parser::parse(&NamedSource::new("test", text.into()))?)
}

macro_rules! literal {
    (String($string:expr), $span:expr) => {
        Expr::lit(Literal::String($string.to_string()), $span)
    };

    ($literal:ident, $span:expr) => {
        Expr::lit(Literal::$literal, $span)
    };
    ($literal:ident($value:expr), $span:expr) => {
        Expr::lit(Literal::$literal($value), $span)
    };
}

macro_rules! box_literal {
    (String($string:expr), $span:expr) => {
        Expr::boxed_lit(Literal::String($string.to_string()), $span)
    };

    ($literal:ident, $span:expr) => {
        Expr::boxed_lit(Literal::$literal, $span)
    };
    ($literal:ident($value:expr), $span:expr) => {
        Expr::boxed_lit(Literal::$literal($value), $span)
    };
}

#[test]
fn boolean() {
    let input = "true";
    let expected = literal!(Bool(true), span(0, 4));
    assert_eq!(parse(input).unwrap(), expected);

    let input = "false";
    let expected = literal!(Bool(false), span(0, 5));
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn number() {
    let input = "42";
    let expected = literal!(Int(42), span(0, 2));
    assert_eq!(parse(input).unwrap(), expected);
}

#[allow(clippy::approx_constant)]
#[test]
fn float() {
    let input = "3.14";
    let expected = literal!(Float(3.14), span(0, 4));
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn string() {
    let input = "\"Hello, world!\"";
    let expected = literal!(String("Hello, world!".to_string()), span(0, 15));
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn escaped_string() {
    let input = "\"Hello, \\n\\tworld!\"";
    let expected = literal!(String("Hello, \n\tworld!".to_string()), span(0, 19));
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn interpolated_string() {
    let input = "\"Hello, my name is ${name}!\"";
    let expected = literal!(
        InterpolatedString(vec![
            Expr::new(
                ExprKind::Literal(Literal::String("Hello, my name is ".to_string())),
                span(1, 18)
            ),
            Expr::new(ExprKind::Identifier("name".to_string()), span(21, 4)),
            Expr::new(
                ExprKind::Literal(Literal::String("!".to_string())),
                span(26, 1)
            ),
        ]),
        span(0, 28)
    );
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn relative_path() {
    let input = "./file.txt";
    let expected = literal!(Path(PathBuf::from("./file.txt")), span(0, 10));
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn absolute_path() {
    let input = "/bin/sh";
    let expected = literal!(Path(PathBuf::from("/bin/sh")), span(0, 7));
    assert_eq!(parse(input).unwrap(), expected);
}

// TODO: Add interpolated path test

#[test]
fn object() {
    let input = "{ name = \"John Doe\" age = 42 }";
    let expected = literal!(
        Object(
            #[rustfmt::skip]
            BTreeMap::from([
                (
                    "name".to_string(),
                    Expr::new(ExprKind::Literal(Literal::String("John Doe".to_string())), span(9, 10))
                ),
                (
                    "age".to_string(),
                    Expr::new(ExprKind::Literal(Literal::Int(42)), span(26, 2))
                ),
            ])
        ),
        span(0, 30)
    );
    assert_eq!(parse(input).unwrap(), expected);

    // TODO: Fix this syntax in parser
    // let input = "{ foo.bar = 3 }";
    // let expected = literal!(
    //     Object(BTreeMap::from([(
    //         "foo".to_string(),
    //         Expr::new(
    //             ExprKind::Literal(Literal::Object(BTreeMap::from([(
    //                 "bar".to_string(),
    //                 Expr::new(
    //                     ExprKind::Literal(Literal::Int(3)),
    //                     Span::new(0..=0, 12..=12)
    //                 )
    //             )]))),
    //             Span::new(0..=0, 7..=13)
    //         )
    //     )])),
    //     Span::new(0..=0, 0..=15)
    // );
    // assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn field_access() {
    let input = "package.dependencies";
    let expected = Expr::new(
        ExprKind::ObjectAccess {
            base: Expr::boxed(ExprKind::Identifier("package".into()), span(0, 7)),
            field: "dependencies".into(),
        },
        span(0, 20),
    );
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn array() {
    let input = "[ 1 2 3 ]";
    let expected = literal!(
        Array(vec![
            literal!(Int(1), span(2, 1)),
            literal!(Int(2), span(4, 1)),
            literal!(Int(3), span(6, 1)),
        ]),
        span(0, 9)
    );
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn not() {
    let input = "!true";
    let expected = Expr::new(
        ExprKind::Not(box_literal!(Bool(true), span(1, 4))),
        span(0, 5),
    );
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn function_declaration() {
    // No arguments
    let input = r#"() { println("Hello!") }"#;
    let expected = Expr::new(
        ExprKind::FnDecl {
            args: vec![],
            expr: Expr::boxed(
                ExprKind::Call {
                    base: Expr::boxed_ident("println", span(5, 7)),
                    args: vec![literal!(String("Hello!".to_string()), span(13, 8))],
                },
                span(5, 17),
            ),
        },
        span(0, 24),
    );
    assert_eq!(parse(input).unwrap(), expected);

    // Single argument
    let input = r#"(name) { "Hello, ${name}!" }"#;
    let expected = Expr::new(
        ExprKind::FnDecl {
            args: vec!["name".to_string()],
            expr: box_literal!(
                InterpolatedString(vec![
                    literal!(String("Hello, ".to_string()), span(10, 7)),
                    Expr::ident("name", span(19, 4)),
                    literal!(String("!".to_string()), span(24, 1)),
                ]),
                span(9, 17)
            ),
        },
        span(0, 28),
    );
    assert_eq!(parse(input).unwrap(), expected);

    // Multiple arguments
    let input = r#"(name, age) { "Hello, ${name}! You are ${age} years old." }"#;
    let expected = Expr::new(
        ExprKind::FnDecl {
            args: vec!["name".to_string(), "age".to_string()],
            expr: box_literal!(
                InterpolatedString(vec![
                    literal!(String("Hello, "), span(15, 7)),
                    Expr::ident("name", span(24, 4)),
                    literal!(String("! You are "), span(30, 10)),
                    Expr::ident("age", span(41, 3)),
                    literal!(String(" years old."), span(45, 11)),
                ]),
                span(14, 43)
            ),
        },
        span(0, 59),
    );
    assert_eq!(parse(input).unwrap(), expected);

    // Complex (declaration + call)
    let input = r"let
    pow = (base, exponent) {
        if(
            exponent == 0,
            1,
            base * pow(base, exponent - 1)
        )
    }
in
    pow(2, 10)";
    let expected = Expr::new(
        ExprKind::LetIn {
            bindings: vec![(
                "pow".to_string(),
                Expr::new(
                    ExprKind::FnDecl {
                        args: vec!["base".to_string(), "exponent".to_string()],
                        expr: Expr::boxed(
                            ExprKind::Call {
                                base: Expr::boxed_ident("if", span(41, 2)),
                                args: vec![
                                    Expr::new(
                                        ExprKind::BinaryOp {
                                            left: Expr::boxed_ident("exponent", span(57, 8)),
                                            operator: BinaryOperator::Eq,
                                            right: box_literal!(Int(0), span(69, 1)),
                                        },
                                        span(57, 13),
                                    ),
                                    literal!(Int(1), span(84, 1)),
                                    Expr::new(
                                        ExprKind::BinaryOp {
                                            left: Expr::boxed_ident("base", span(99, 4)),
                                            operator: BinaryOperator::Multiply,
                                            right: Expr::boxed(
                                                ExprKind::Call {
                                                    base: Expr::boxed_ident("pow", span(106, 3)),
                                                    args: vec![
                                                        Expr::ident("base", span(110, 4)),
                                                        Expr::new(
                                                            ExprKind::BinaryOp {
                                                                left: Expr::boxed_ident(
                                                                    "exponent",
                                                                    span(116, 8),
                                                                ),
                                                                operator: BinaryOperator::Minus,
                                                                right: box_literal!(
                                                                    Int(1),
                                                                    span(127, 1)
                                                                ),
                                                            },
                                                            span(116, 12),
                                                        ),
                                                    ],
                                                },
                                                span(106, 23),
                                            ),
                                        },
                                        span(99, 30),
                                    ),
                                ],
                            },
                            span(41, 98),
                        ),
                    },
                    span(14, 131),
                ),
            )],
            expr: Expr::boxed(
                ExprKind::Call {
                    base: Expr::boxed_ident("pow", span(153, 3)),
                    args: vec![
                        literal!(Int(2), span(157, 1)),
                        literal!(Int(10), span(160, 2)),
                    ],
                },
                span(153, 10),
            ),
        },
        span(0, 163),
    );
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn call() {
    // No arguments
    let input = "exit()";
    let expected = Expr::new(
        ExprKind::Call {
            base: Expr::boxed_ident("exit", span(0, 4)),
            args: vec![],
        },
        span(0, 6),
    );
    assert_eq!(parse(input).unwrap(), expected);

    // Single argument
    let input = "println(\"Hello, world!\")";
    let expected = Expr::new(
        ExprKind::Call {
            base: Expr::boxed_ident("println", span(0, 7)),
            args: vec![literal!(String("Hello, world!"), span(8, 15))],
        },
        span(0, 24),
    );
    assert_eq!(parse(input).unwrap(), expected);

    // Multiple arguments
    let input = r#"if(
    password == "strongpassword123"
    "Password is correct"
    "Password is incorrect"
)"#;
    let expected = Expr::new(
        ExprKind::Call {
            base: Expr::boxed_ident("if", span(0, 2)),
            args: vec![
                Expr::new(
                    ExprKind::BinaryOp {
                        left: Expr::boxed_ident("password", span(8, 8)),
                        operator: BinaryOperator::Eq,
                        right: box_literal!(String("strongpassword123"), span(20, 19)),
                    },
                    span(8, 31),
                ),
                literal!(String("Password is correct"), span(44, 21)),
                literal!(String("Password is incorrect"), span(70, 23)),
            ],
        },
        span(0, 95),
    );
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn binary_op() {
    let input = "2 + 3 * 4";
    let expected = Expr::new(
        ExprKind::BinaryOp {
            left: box_literal!(Int(2), span(0, 1)),
            operator: BinaryOperator::Plus,
            right: Expr::boxed(
                ExprKind::BinaryOp {
                    left: box_literal!(Int(3), span(4, 1)),
                    operator: BinaryOperator::Multiply,
                    right: box_literal!(Int(4), span(8, 1)),
                },
                span(4, 5),
            ),
        },
        span(0, 9),
    );
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn bindings() {
    let input = r#"let
    name = "John Doe"
in
    null
"#;
    let expected = Expr::new(
        ExprKind::LetIn {
            bindings: vec![(
                "name".to_string(),
                literal!(String("John Doe".to_string()), span(15, 10)),
            )],
            expr: box_literal!(Null, span(33, 4)),
        },
        span(0, 37),
    );
    assert_eq!(parse(input).unwrap(), expected);
}
