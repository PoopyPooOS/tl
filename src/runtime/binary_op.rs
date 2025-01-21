use super::{types::Value, ValueResult};
use crate::parser::ast::types::{BinaryOperator, Expr};

#[allow(clippy::todo, reason = "This is not production code")]
impl super::Scope {
    pub(super) fn eval_binary_op(
        &mut self,
        left: &Expr,
        operator: &BinaryOperator,
        right: &Expr,
    ) -> ValueResult {
        let left = self.eval_expr(left)?;
        let right = self.eval_expr(right)?;

        #[allow(
            clippy::arithmetic_side_effects,
            reason = "Arthimetic operation implementations for `Value` uses saturating ops where it can."
        )]
        Ok(match operator {
            BinaryOperator::Plus => left + right,
            BinaryOperator::Minus => left - right,
            BinaryOperator::Multiply => left * right,
            BinaryOperator::Divide => left / right,
            BinaryOperator::Modulo => left % right,
            BinaryOperator::Eq => Value::Boolean(left == right),
            BinaryOperator::NotEq => Value::Boolean(left != right),
            BinaryOperator::Gt => Value::Boolean(left > right),
            BinaryOperator::GtEq => Value::Boolean(left >= right),
            BinaryOperator::Lt => Value::Boolean(left < right),
            BinaryOperator::LtEq => Value::Boolean(left <= right),
            BinaryOperator::And => Value::Boolean(left.and(&right)),
            BinaryOperator::Or => Value::Boolean(left.or(&right)),
            BinaryOperator::Dot => todo!("Implement field access"),
        })
    }
}
