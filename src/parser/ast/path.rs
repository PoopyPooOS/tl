use super::ExprResult;
use crate::parser::{
    ast::{
        consume, raw_err,
        types::{Expr, ExprType, Literal},
    },
    tokenizer::types::TokenType,
};
use logger::location::Section;
use std::path::PathBuf;

impl super::Parser {
    pub(super) fn parse_path(&mut self) -> ExprResult {
        let start = self
            .tokens
            .get(self.position)
            .ok_or_else(|| raw_err!(ExpectedOneOfTokens(vec![TokenType::Dot, TokenType::Slash])))?;

        // consume `./`
        if consume!(no_propagate self, Dot).is_ok() {
            consume!(self, Slash);
        }

        let mut path = PathBuf::new();
        if start.token_type == TokenType::Slash {
            path.push("/");
        }

        let mut current_path_component = String::new();

        while let Some(next_token) = self.tokens.get(self.position) {
            match &next_token.token_type {
                TokenType::Slash => {
                    path.push(&current_path_component);
                    current_path_component.clear();
                    self.position = self.position.saturating_add(1);
                    continue;
                }
                TokenType::Identifier(v) => current_path_component.push_str(v),
                TokenType::Dot => current_path_component.push('.'),
                _ => break,
            }

            self.position = self.position.saturating_add(1);
        }

        if !current_path_component.is_empty() {
            path.push(current_path_component);
        }

        let section = Section::merge_start_end(
            &start.section,
            &self
                .tokens
                .get(self.position.saturating_sub(1))
                .unwrap_or(start)
                .section,
        );

        Ok(Expr::new(ExprType::Literal(Literal::Path(path)), section))
    }
}
