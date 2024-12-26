use super::{
    types::{Error, ErrorType, Expr, ExprType, Literal},
    ExprResult,
};
use crate::parser::tokenizer::types::TokenType;

impl super::Parser {
    pub(super) fn parse_array(&mut self) -> ExprResult {
        let start = self
            .tokens
            .get(self.position)
            .ok_or_else(|| Box::new(Error::new(ErrorType::ExpectedToken(TokenType::LBracket), None)))?
            .clone();
        self.consume(TokenType::LBracket)?;

        let mut array = Vec::new();
        while let Some(next_token) = self.tokens.get(self.position).cloned() {
            if next_token.token_type == TokenType::RBracket {
                self.consume(TokenType::RBracket)?;
                break;
            }

            let expr = self.parse_expr()?;
            array.push(expr);
        }

        let end = self.tokens.get(self.position.saturating_sub(1)).unwrap_or(&start).cols.end();
        Ok(Expr::new(
            ExprType::Literal(Literal::Array(array)),
            start.line,
            *start.cols.start()..=*end,
        ))
    }
}
