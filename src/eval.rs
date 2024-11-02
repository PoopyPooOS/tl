use crate::ast::Expr;

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn eval(&mut self, expr: Expr) {
        match expr {
            Expr::Literal(literal_expr) => println!("eval: {:?}", literal_expr.value),
            Expr::Identifier(a) => println!("eval: {a}"),
            Expr::BinaryOp(binary_op_expr) => todo!(),
            Expr::UnaryOp(unary_op_expr) => todo!(),
            Expr::FunctionCall(function_call_expr) => todo!(),
            Expr::Lambda(lambda_expr) => todo!(),
            Expr::Conditional(conditional_expr) => todo!(),
            Expr::Array(array_expr) => todo!(),
            Expr::Object(object_expr) => todo!(),
        }
    }
}
