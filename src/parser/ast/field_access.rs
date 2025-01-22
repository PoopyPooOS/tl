use super::ExprResult;
use crate::parser::{
    ast::{
        get_ident,
        types::{Expr, ExprType},
    },
    tokenizer::types::TokenType,
};
use logger::location::Section;

impl super::Parser {
    pub(super) fn parse_field_access(&mut self) -> ExprResult {
        self.position = self.position.saturating_sub(1);
        let base = get_ident!(self, true);
        let base_expr = Expr::new(ExprType::Identifier(base.1), base.0.section.clone());
        let mut path: Vec<Expr> = Vec::new();

        let mut seperated = false;

        while let Some(next_token) = self.tokens.get(self.position) {
            match &next_token.token_type {
                TokenType::Dot => seperated = true,
                TokenType::Identifier(v) if seperated => {
                    seperated = false;

                    path.push(Expr::new(
                        ExprType::Identifier(v.clone()),
                        next_token.section.clone(),
                    ));
                }
                _ => break,
            }

            self.position = self.position.saturating_add(1);
        }

        let section =
            Section::merge_start_end(&base.0.section, &path.last().unwrap_or(&base_expr).section);
        Ok(Expr::new(
            ExprType::FieldAccess {
                base: Box::new(base_expr),
                path,
            },
            section,
        ))
    }
}
