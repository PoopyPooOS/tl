use crate::parser::ast::types::Expr;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Null,
    Int(isize),
    Float(f64),
    Bool(bool),
    String(String),
    InterpolatedString(Vec<Expr>),
    Path(PathBuf),
    InterpolatedPath(Vec<Expr>),
    Array(Vec<Expr>),
    Object(Vec<(Expr, Expr)>),
}
