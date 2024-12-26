use super::{
    types::{BinaryOperator, Error, ErrorType, Expr, ExprType},
    ExprResult,
};
use crate::parser::tokenizer::types::TokenType;

impl super::Parser {
    pub(super) fn parse_binary_op(&mut self, min_precedence: u8) -> ExprResult {
        let start = self
            .tokens
            .get(self.position)
            .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?
            .clone();

        let mut left = if start.token_type == TokenType::LParen {
            self.consume(TokenType::LParen)?;
            let expr = self.parse_binary_op(0)?;
            self.consume(TokenType::RParen)?;
            expr
        } else {
            self.parse_literal()?
        };

        while let Some(next_token) = self.tokens.get(self.position) {
            if !next_token.token_type.is_binary_operator() {
                break;
            }

            let operator_token = self
                .tokens
                .get(self.position)
                .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?
                .clone();

            self.position = self.position.saturating_add(1);
            let operator = BinaryOperator::from_token(operator_token.token_type.clone())?;
            let precedence = operator.precedence();

            if precedence < min_precedence {
                break;
            }

            if self.tokens.get(self.position).is_none() {
                return Err(Box::new(Error::new(
                    ErrorType::MissingRightSide,
                    self.location_from_token(&operator_token),
                )));
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
}
