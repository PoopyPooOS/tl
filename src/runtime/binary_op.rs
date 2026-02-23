use crate::{
    merge_spans,
    parser::ast::types::{BinaryOperator, Expr},
    runtime::{Value, ValueKind, types::ValueResult},
};

impl super::Scope {
    pub(super) fn eval_binary_op(
        &self,
        left: &Expr,
        operator: &BinaryOperator,
        right: &Expr,
    ) -> ValueResult {
        let lhs = self.eval_expr(left)?;
        let rhs = self.eval_expr(right)?;

        #[allow(
            clippy::arithmetic_side_effects,
            reason = "Arithmetic operation implementations for `Value` uses saturating ops where it can."
        )]
        Ok(match operator {
            BinaryOperator::Plus => lhs + rhs,
            BinaryOperator::Minus => lhs - rhs,
            BinaryOperator::Multiply => lhs * rhs,
            BinaryOperator::Divide => lhs / rhs,
            BinaryOperator::Modulo => lhs % rhs,
            BinaryOperator::Eq => {
                Value::new(ValueKind::Bool(lhs == rhs), merge_spans(lhs.span, rhs.span))
            }
            BinaryOperator::NotEq => {
                Value::new(ValueKind::Bool(lhs != rhs), merge_spans(lhs.span, rhs.span))
            }
            BinaryOperator::Gt => {
                Value::new(ValueKind::Bool(lhs > rhs), merge_spans(lhs.span, rhs.span))
            }
            BinaryOperator::GtEq => {
                Value::new(ValueKind::Bool(lhs >= rhs), merge_spans(lhs.span, rhs.span))
            }
            BinaryOperator::Lt => {
                Value::new(ValueKind::Bool(lhs < rhs), merge_spans(lhs.span, rhs.span))
            }
            BinaryOperator::LtEq => {
                Value::new(ValueKind::Bool(lhs <= rhs), merge_spans(lhs.span, rhs.span))
            }
            BinaryOperator::And => Value::new(
                ValueKind::Bool(lhs.and(&rhs)),
                merge_spans(lhs.span, rhs.span),
            ),
            BinaryOperator::Or => Value::new(
                ValueKind::Bool(lhs.or(&rhs)),
                merge_spans(lhs.span, rhs.span),
            ),
        })
    }
}
