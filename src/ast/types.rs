#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Statement {
    Let { name: String, value: Expr },
    Expr(Expr),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp(BinaryOp),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Literal {
    Number(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct BinaryOp {
    left: &'static Expr,
    operator: BinaryOperator,
    right: &'static Expr,
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
}
