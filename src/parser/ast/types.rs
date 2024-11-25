#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Statement {
    Let { name: String, value: Expr },
    Call { name: String, args: Vec<Expr> },
    Expr(Expr),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Literal {
    Number(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
}
