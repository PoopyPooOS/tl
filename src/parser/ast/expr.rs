use super::{
    types::{Error, ErrorType, Expr, ExprType, Literal},
    ExprResult,
};
use crate::parser::tokenizer::types::TokenType;
use logger::Location;

impl super::Parser {
    pub(super) fn parse_expr(&mut self) -> ExprResult {
        if let Some(next_token) = self.tokens.get(self.position.saturating_add(1)) {
            if next_token.token_type.is_binary_operator() {
                return self.parse_binary_op(0);
            }
        }

        let token = self
            .tokens
            .get(self.position)
            .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?;

        let expr = match token.token_type {
            TokenType::LBrace => Some(self.parse_object()?),
            TokenType::LBracket => Some(self.parse_array()?),
            TokenType::Identifier(_) => Some(self.parse_ident()?),
            TokenType::LParen => {
                if let Some(next_token) = self.tokens.get(self.position.saturating_add(1)) {
                    let next_next_token = self.tokens.get(self.position.saturating_add(2));

                    // Function Declaration
                    if matches!(next_token.token_type, TokenType::Identifier(_) | TokenType::RParen) {
                        return self.parse_fn_decl();
                    }

                    // Binary Operation
                    if next_token.token_type.is_number() || next_next_token.is_some_and(|token| token.token_type.is_binary_operator()) {
                        return self.parse_binary_op(0);
                    }
                }

                None
            }
            TokenType::Not => {
                let token = self
                    .tokens
                    .get(self.position)
                    .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?
                    .clone();

                self.consume(TokenType::Not)?;
                let expr = self.parse_expr()?;
                let end = *expr.cols.end();
                Some(Expr::new(ExprType::Not(Box::new(expr)), token.line, *token.cols.start()..=end))
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
            .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?
            .clone();

        let expr = match &token.token_type {
            TokenType::Null => Expr::new(ExprType::Literal(Literal::Null), token.line, token.cols),
            TokenType::String(v) => Expr::new(ExprType::Literal(Literal::String(v.clone())), token.line, token.cols),
            TokenType::InterpolatedString(v) => self.parse_interpolated_string(v)?,
            TokenType::Int(v) => Expr::new(ExprType::Literal(Literal::Int(*v)), token.line, token.cols),
            TokenType::Float(v) => Expr::new(ExprType::Literal(Literal::Float(*v)), token.line, token.cols),
            TokenType::Bool(v) => Expr::new(ExprType::Literal(Literal::Bool(*v)), token.line, token.cols),
            TokenType::Identifier(v) => {
                if let Some(next_token) = self.tokens.get(self.position.saturating_add(1))
                    && next_token.token_type == TokenType::LParen
                {
                    let expr = self.parse_ident()?;

                    if matches!(expr.expr_type, ExprType::Call { .. }) {
                        // This is to not skip tokens.
                        self.position = self.position.saturating_sub(1);
                    }

                    expr
                } else {
                    Expr::new(ExprType::Identifier(v.clone()), token.line, token.cols)
                }
            }
            other => {
                let location = Location {
                    path: self.source.path.clone(),
                    text: self.source.text.clone(),
                    lines: token.line..=token.line,
                    section: Some(token.cols),
                };

                return Err(Box::new(Error::new(ErrorType::UnexpectedToken(other.clone()), Some(location))));
            }
        };

        self.position = self.position.saturating_add(1);

        Ok(expr)
    }
}
