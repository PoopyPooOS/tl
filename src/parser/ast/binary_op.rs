use super::{
    err, raw_err,
    types::{BinaryOperator, Expr, ExprType},
    ExprResult,
};
use crate::parser::{ast::consume, tokenizer::types::TokenType};

impl super::Parser {
    fn parse_binary_op(&mut self, min_precedence: u8) -> ExprResult {
        let start = self
            .tokens
            .get(self.position)
            .ok_or_else(|| raw_err!(NoTokensLeft))?
            .clone();

        let mut left = if start.token_type == TokenType::LParen {
            consume!(self, LParen);
            let expr = self.parse_binary_op(0)?;
            consume!(self, RParen);
            expr
        } else {
            self.parse_literal()?
        };

        if self
            .tokens
            .get(self.position.saturating_add(1))
            .is_none_or(|token| !token.token_type.is_binary_operator())
        {
            return Ok(left);
        }

        while let Some(next_token) = self.tokens.get(self.position) {
            if !next_token.token_type.is_binary_operator() {
                break;
            }

            let operator_token = self
                .tokens
                .get(self.position)
                .ok_or_else(|| raw_err!(NoTokensLeft))?
                .clone();

            self.position = self.position.saturating_add(1);
            let operator = BinaryOperator::from_token(operator_token.token_type.clone())?;
            let precedence = operator.precedence();

            if precedence < min_precedence {
                break;
            }

            if self.tokens.get(self.position).is_none() {
                return err!(MissingRightSide, self.location_from_token(&operator_token));
            }

            let right = self.parse_binary_op(precedence.saturating_add(1))?;

            let line = left.line;
            let end = *right.cols.end();
            left = Expr::new(
                ExprType::BinaryOp {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                line,
                *start.cols.start()..=end,
            );
        }

        Ok(left)
    }

    pub(super) fn parse_binary_op_with_left(
        &mut self,
        min_precedence: u8,
        mut left: Expr,
    ) -> ExprResult {
        if self
            .tokens
            .get(self.position)
            .is_none_or(|token| !token.token_type.is_binary_operator())
        {
            return Ok(left);
        }

        while let Some(next_token) = self.tokens.get(self.position) {
            if !next_token.token_type.is_binary_operator() {
                break;
            }

            let operator_token = self
                .tokens
                .get(self.position)
                .ok_or_else(|| raw_err!(NoTokensLeft))?
                .clone();

            self.position = self.position.saturating_add(1);
            let operator = BinaryOperator::from_token(operator_token.token_type.clone())?;
            let precedence = operator.precedence();

            if precedence < min_precedence {
                break;
            }

            if self.tokens.get(self.position).is_none() {
                return err!(MissingRightSide, self.location_from_token(&operator_token));
            }

            let right = self.parse_binary_op(precedence.saturating_add(1))?;

            let start = *left.cols.start();
            let line = left.line;
            let end = *right.cols.end();
            left = Expr::new(
                ExprType::BinaryOp {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                line,
                start..=end,
            );
        }

        Ok(left)
    }
}
