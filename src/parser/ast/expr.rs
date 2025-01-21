use super::{
    err, raw_err,
    types::{Expr, ExprType, Literal},
    ExprResult,
};
use crate::parser::{ast::consume, tokenizer::types::TokenType};
use logger::{location::Section, Location};

impl super::Parser {
    pub(super) fn parse_expr(&mut self) -> ExprResult {
        let token = self
            .tokens
            .get(self.position)
            .ok_or_else(|| raw_err!(NoTokensLeft))?;

        let expr = match token.token_type {
            TokenType::LBrace => Some(self.parse_object()?),
            TokenType::LBracket => Some(self.parse_array()?),
            TokenType::LParen => {
                // Function Declaration
                if let Some(next_token) = self.tokens.get(self.position.saturating_add(1))
                    && matches!(
                        next_token.token_type,
                        TokenType::Identifier(_) | TokenType::RParen
                    )
                {
                    return self.parse_fn_decl();
                }

                None
            }
            TokenType::Not => {
                let token = self
                    .tokens
                    .get(self.position)
                    .ok_or_else(|| raw_err!(NoTokensLeft))?
                    .clone();

                consume!(self, Not);
                let expr = self.parse_expr()?;
                let section = Section::merge_start_end(&token.section, &expr.section);

                Some(Expr::new(ExprType::Not(Box::new(expr)), section))
            }
            _ => None,
        };

        if let Some(expr) = expr {
            return Ok(expr);
        }

        self.parse_literal()
    }

    pub(super) fn parse_literal(&mut self) -> ExprResult {
        let token = self
            .tokens
            .get(self.position)
            .ok_or_else(|| raw_err!(NoTokensLeft))?
            .clone();

        macro_rules! literal {
            ($variant:ident) => {{
                self.position = self.position.saturating_add(1);
                Expr::new(ExprType::Literal(Literal::$variant), token.section)
            }};
            ($variant:ident($value:expr)) => {{
                self.position = self.position.saturating_add(1);
                Expr::new(ExprType::Literal(Literal::$variant($value)), token.section)
            }};
        }

        let expr = match &token.token_type {
            TokenType::Null => literal!(Null),
            TokenType::String(v) => literal!(String(v.clone())),
            TokenType::InterpolatedString(v) => self.parse_interpolated_string(v)?,
            TokenType::Int(v) => literal!(Int(*v)),
            TokenType::Float(v) => literal!(Float(*v)),
            TokenType::Bool(v) => literal!(Boolean(*v)),
            TokenType::Identifier(_) => self.parse_ident()?,
            other => {
                let location = Location {
                    path: self.source.path.clone(),
                    text: self.source.text.clone(),
                    section: Some(token.section),
                };

                return err!(UnexpectedToken(other.clone()), Some(location));
            }
        };

        let token = self.tokens.get(self.position);
        if token.is_some_and(|token| token.token_type.is_binary_operator()) {
            return self.parse_binary_op_with_left(0, expr);
        }

        Ok(expr)
    }
}
