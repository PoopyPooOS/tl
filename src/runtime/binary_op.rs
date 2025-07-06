use super::{types::Value, ValueResult};
use crate::parser::ast::types::{BinaryOperator, Expr};

impl super::Scope {
    pub(super) fn eval_binary_op(
        &mut self,
        left: &Expr,
        operator: &BinaryOperator,
        right: &Expr,
    ) -> ValueResult {
        let lhs = self.eval_expr(left)?;

        // // Do field access before evaluating rhs because rhs would not be a valid variable.
        // if operator == &BinaryOperator::Dot {
        //     match &right.expr_type {
        //         ExprType::Identifier(v) => return Ok(lhs.access(v)),
        //         ExprType::BinaryOp {
        //             left,
        //             operator,
        //             right,
        //         } if operator == &BinaryOperator::Dot => {
        //             return self.eval_binary_op(left, operator, right)
        //         }
        //         _ => {
        //             return Err(Box::new(Error::new(
        //                 ErrorType::CanNotAccessWithNonIdent,
        //                 self.location_from_expr(right),
        //             )))
        //         }
        //     }
        // }

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
            BinaryOperator::Eq => Value::Boolean(lhs == rhs),
            BinaryOperator::NotEq => Value::Boolean(lhs != rhs),
            BinaryOperator::Gt => Value::Boolean(lhs > rhs),
            BinaryOperator::GtEq => Value::Boolean(lhs >= rhs),
            BinaryOperator::Lt => Value::Boolean(lhs < rhs),
            BinaryOperator::LtEq => Value::Boolean(lhs <= rhs),
            BinaryOperator::And => Value::Boolean(lhs.and(&rhs)),
            BinaryOperator::Or => Value::Boolean(lhs.or(&rhs)),
        })
    }
}
