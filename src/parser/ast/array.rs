use super::{
    types::{Expr, ExprType, Literal},
    ExprResult,
};
use crate::parser::{
    ast::{consume, raw_err},
    tokenizer::types::TokenType,
};
use logger::location::Section;

impl super::Parser {
    pub(super) fn parse_array(&mut self) -> ExprResult {
        let start = self
            .tokens
            .get(self.position)
            .ok_or_else(|| raw_err!(ExpectedToken(TokenType::LBracket)))?
            .clone();
        consume!(self, LBracket);

        let mut array = Vec::new();
        while let Some(next_token) = self.tokens.get(self.position).cloned() {
            if next_token.token_type == TokenType::RBracket {
                consume!(self, RBracket);
                break;
            }

            let expr = self.parse_expr()?;
            array.push(expr);
        }

        let end = self
            .tokens
            .get(self.position.saturating_sub(1))
            .unwrap_or(&start);

        Ok(Expr::new(
            ExprType::Literal(Literal::Array(array)),
            Section::merge_start_end(&start.section, &end.section),
        ))
    }
}
