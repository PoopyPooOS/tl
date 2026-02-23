use crate::{
    merge_spans,
    parser::ast::{
        ExprResult,
        types::{BinaryOperator, Error, ErrorKind, Expr, ExprKind},
    },
};
use tl_macro::{change_pos, peek};

impl super::Parser {
    pub(super) fn parse_binary_op_with_left(
        &mut self,
        min_precedence: u8,
        mut left: Expr,
    ) -> ExprResult {
        while let Some(operator_token) = peek!(0)
            && operator_token.kind.is_binary_operator()
        {
            let operator = BinaryOperator::from_token(operator_token.kind.clone())?;
            let precedence = operator.precedence();
            if precedence < min_precedence {
                break;
            }

            change_pos!(1);

            if peek!(0).is_none() {
                return Err(Error::new(
                    ErrorKind::MissingRightSide,
                    self.source.clone(),
                    merge_spans(left.span, operator_token.span),
                ));
            }

            let right = self.parse_binary_op(precedence.saturating_add(1))?;
            let span = merge_spans(left.span, right.span);

            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn parse_binary_op(&mut self, min_precedence: u8) -> ExprResult {
        let left = self.parse_primary()?;
        let left = self.parse_expr_suffixes(left)?;
        self.parse_binary_op_with_left(min_precedence, left)
    }
}
