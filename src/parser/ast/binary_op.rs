use super::{
    ExprResult,
    types::{BinaryOperator, Expr, ExprKind},
};
use crate::{
    merge_spans,
    parser::ast::types::{Error, ErrorKind},
};

impl super::Parser {
    pub(super) fn parse_binary_op_with_left(
        &mut self,
        min_precedence: u8,
        mut left: Expr,
    ) -> ExprResult {
        if self
            .tokens
            .get(self.pos)
            .is_none_or(|token| !token.kind.is_binary_operator())
        {
            return Ok(left);
        }

        while let Some(next_token) = self.tokens.get(self.pos) {
            if !next_token.kind.is_binary_operator() {
                break;
            }

            let operator_token = self
                .tokens
                .get(self.pos)
                .ok_or(Error::new(
                    ErrorKind::NoTokensLeft,
                    self.source.clone(),
                    self.closest_span(),
                ))?
                .clone();

            self.pos = self.pos.saturating_add(1);
            let operator = BinaryOperator::from_token(operator_token.kind.clone())?;
            let precedence = operator.precedence();

            if precedence < min_precedence {
                break;
            }

            if self.tokens.get(self.pos).is_none() {
                return Err(Error::new(
                    ErrorKind::MissingRightSide,
                    self.source.clone(),
                    operator_token.span,
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
        let start = self
            .tokens
            .get(self.pos)
            .ok_or(Error::new(
                ErrorKind::NoTokensLeft,
                self.source.clone(),
                self.closest_span(),
            ))?
            .clone();

        let mut left = self.parse_literal()?;

        if self
            .tokens
            .get(self.pos.saturating_add(1))
            .is_none_or(|token| !token.kind.is_binary_operator())
        {
            return Ok(left);
        }

        while let Some(next_token) = self.tokens.get(self.pos) {
            if !next_token.kind.is_binary_operator() {
                break;
            }

            let operator_token = self
                .tokens
                .get(self.pos)
                .ok_or(Error::new(
                    ErrorKind::NoTokensLeft,
                    self.source.clone(),
                    self.closest_span(),
                ))?
                .clone();

            self.pos = self.pos.saturating_add(1);
            let operator = BinaryOperator::from_token(operator_token.kind.clone())?;
            let precedence = operator.precedence();

            if precedence < min_precedence {
                break;
            }

            if self.tokens.get(self.pos).is_none() {
                return Err(Error::new(
                    ErrorKind::MissingRightSide,
                    self.source.clone(),
                    operator_token.span,
                ));
            }

            let right = self.parse_binary_op(precedence.saturating_add(1))?;
            let span = merge_spans(start.span, right.span);

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
}
