use std::{collections::HashMap, fmt::Display};

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
    InterpolatedString(Vec<Expr>),
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

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Plus => write!(f, "+"),
            BinaryOperator::Minus => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
        }
    }
}
