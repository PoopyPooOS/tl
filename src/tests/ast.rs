use crate::{
    ast::{
        types::{BinaryOperator, Expr, Literal, Statement},
        Parser,
    },
    tokenizer::types::Token,
};

fn parse(tokens: impl Into<Vec<Token>>) -> Vec<Statement> {
    let tokens: Vec<Token> = tokens.into();
    Parser::new(tokens).parse().unwrap()
}

macro_rules! literal {
    ($literal:ident, $value:expr) => {
        Statement::Expr(Expr::Literal(Literal::$literal($value)))
    };
}

#[test]
fn boolean() {
    let input = [Token::Bool(true)];
    let expected = vec![literal!(Bool, true)];
    assert_eq!(parse(input), expected);

    let input = [Token::Bool(false)];
    let expected = vec![literal!(Bool, false)];
    assert_eq!(parse(input), expected);
}

#[test]
fn number() {
    let input = [Token::Number(42)];
    let expected = vec![literal!(Number, 42)];
    assert_eq!(parse(input), expected);
}

#[test]
fn float() {
    const PI: f64 = std::f64::consts::PI;

    let input = [Token::Float(PI)];
    let expected = vec![literal!(Float, PI)];
    assert_eq!(parse(input), expected);
}

#[test]
fn string() {
    let input = [Token::String("Hello, world!".to_string())];
    let expected = vec![literal!(String, "Hello, world!".to_string())];
    assert_eq!(parse(input), expected);
}

#[test]
fn escaped_string() {
    let input = [Token::String("Hello, \n\tworld!".to_string())];
    let expected = vec![literal!(String, "Hello, \n\tworld!".to_string())];
    assert_eq!(parse(input), expected);
}

// Dont try to read this, you will get an aneurysm
#[test]
fn binary_operators() {
    let input = [
        Token::Number(1),
        Token::Plus,
        Token::Number(1),
        Token::Minus,
        Token::Number(1),
        Token::Multiply,
        Token::Number(1),
        Token::Slash,
        Token::Number(1),
    ];
    let expected = vec![Statement::Expr(Expr::BinaryOp {
        left: Box::new(Expr::BinaryOp {
            left: Box::new(Expr::Literal(Literal::Number(1))),
            operator: BinaryOperator::Plus,
            right: Box::new(Expr::Literal(Literal::Number(1))),
        }),
        operator: BinaryOperator::Minus,
        right: Box::new(Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Literal(Literal::Number(1))),
                operator: BinaryOperator::Multiply,
                right: Box::new(Expr::Literal(Literal::Number(1))),
            }),
            operator: BinaryOperator::Divide,
            right: Box::new(Expr::Literal(Literal::Number(1))),
        }),
    })];
    assert_eq!(parse(input), expected);
}

// #[test]
fn round_brackets() {
    let input = [
        Token::LParen,
        Token::Number(2),
        Token::Plus,
        Token::Number(3),
        Token::RParen,
        Token::Multiply,
        Token::Number(4),
    ];
    let expected = vec![literal!(Number, 0)];
    assert_eq!(parse(input), expected);
}
