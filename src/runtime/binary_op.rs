use super::{ValueResult, types::Value};
use crate::{
    merge_spans,
    parser::ast::types::{BinaryOperator, Expr},
    runtime::ValueKind,
};

impl super::Scope {
    pub(super) fn eval_binary_op(
        &mut self,
        left: &Expr,
        operator: &BinaryOperator,
        right: &Expr,
    ) -> ValueResult {
        let lhs = self.eval_expr(left)?;
        let rhs = self.eval_expr(right)?;

        #[allow(
            clippy::arithmetic_side_effects,
            reason = "Arthimetic operation implementations for `Value` uses saturating ops where it can."
        )]
        Ok(match operator {
            BinaryOperator::Plus => lhs + rhs,
            BinaryOperator::Minus => lhs - rhs,
            BinaryOperator::Multiply => lhs * rhs,
            BinaryOperator::Divide => lhs / rhs,
            BinaryOperator::Modulo => lhs % rhs,
            BinaryOperator::Eq => Value::new(
                ValueKind::Boolean(lhs == rhs),
                merge_spans(lhs.span, rhs.span),
            ),
            BinaryOperator::NotEq => Value::new(
                ValueKind::Boolean(lhs != rhs),
                merge_spans(lhs.span, rhs.span),
            ),
            BinaryOperator::Gt => Value::new(
                ValueKind::Boolean(lhs > rhs),
                merge_spans(lhs.span, rhs.span),
            ),
            BinaryOperator::GtEq => Value::new(
                ValueKind::Boolean(lhs >= rhs),
                merge_spans(lhs.span, rhs.span),
            ),
            BinaryOperator::Lt => Value::new(
                ValueKind::Boolean(lhs < rhs),
                merge_spans(lhs.span, rhs.span),
            ),
            BinaryOperator::LtEq => Value::new(
                ValueKind::Boolean(lhs <= rhs),
                merge_spans(lhs.span, rhs.span),
            ),
            BinaryOperator::And => Value::new(
                ValueKind::Boolean(lhs.and(&rhs)),
                merge_spans(lhs.span, rhs.span),
            ),
            BinaryOperator::Or => Value::new(
                ValueKind::Boolean(lhs.or(&rhs)),
                merge_spans(lhs.span, rhs.span),
            ),
        })
    }
}
