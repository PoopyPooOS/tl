use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expr,
    },
    Fn {
        name: String,
        parameters: Vec<String>, // TODO: Parameter struct that contains type information.
        body: Vec<Statement>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Expr(Expr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Number(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Expr>),
    Object(HashMap<String, Expr>),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
}
