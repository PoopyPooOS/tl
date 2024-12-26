use super::{
    types::{Expr, ExprType, Literal},
    ExprResult,
};
use crate::parser::tokenizer::types::TokenType;

impl super::Parser {
    pub(super) fn parse_array(&mut self) -> ExprResult {
        let start = self.tokens[self.position].clone();
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

        let end = &self.tokens[self.position - 1].cols.end();
        Ok(Expr::new(
            ExprType::Literal(Literal::Array(array)),
            start.line,
            *start.cols.start()..=**end,
        ))
    }
}
