use std::path::PathBuf;

use super::{
    ExprResult,
    types::{Expr, ExprKind, Literal},
};
use crate::{
    merge_spans,
    parser::{
        ast::{
            consume,
            types::{Error, ErrorKind},
        },
        lexer::types::TokenKind,
    },
};

impl super::Parser {
    /// Generates an AST based on the tokens of this [`Parser`].
    /// # Errors
    /// This function will return an error if a AST generation error occurs.
    pub fn parse(&mut self) -> ExprResult {
        let token = self.tokens.get(self.pos).ok_or(Error::new(
            ErrorKind::NoTokensLeft,
            self.source.clone(),
            self.closest_span(),
        ))?;

        let expr = match token.kind {
            TokenKind::LBrace => Some(self.parse_object()?),
            TokenKind::LBracket => Some(self.parse_array()?),
            TokenKind::Identifier(_)
                if self
                    .tokens
                    .get(self.pos.saturating_add(1))
                    .is_some_and(|t| t.kind == TokenKind::Colon) =>
            {
                Some(self.parse_fn_decl()?)
            }
            TokenKind::Not => {
                let token = self
                    .tokens
                    .get(self.pos)
                    .ok_or(Error::new(
                        ErrorKind::NoTokensLeft,
                        self.source.clone(),
                        self.closest_span(),
                    ))?
                    .clone();

                consume!(self, Not);
                let expr = self.parse()?;
                let span = merge_spans(token.span, expr.span);

                Some(Expr::new(ExprKind::Not(Box::new(expr)), span))
            }
            TokenKind::Let => Some(self.parse_let()?),
            _ => None,
        };

        if let Some(expr) = expr {
            return Ok(expr);
        }

        let expr = self.parse_primary()?;
        let token = self.tokens.get(self.pos);

        if let Some(token) = token {
            match &token.kind {
                b if b.is_binary_operator() => {
                    return self.parse_binary_op_with_left(0, expr);
                }
                _ => (),
            }
        }

        Ok(expr)
    }

    pub(super) fn parse_primary(&mut self) -> ExprResult {
        let token = self
            .tokens
            .get(self.pos)
            .ok_or(Error::new(
                ErrorKind::NoTokensLeft,
                self.source.clone(),
                self.closest_span(),
            ))?
            .clone();

        macro_rules! literal {
            ($variant:ident) => {{
                self.pos = self.pos.saturating_add(1);
                Expr::new(ExprKind::Literal(Literal::$variant), token.span)
            }};
            ($variant:ident($value:expr)) => {{
                self.pos = self.pos.saturating_add(1);
                Expr::new(ExprKind::Literal(Literal::$variant($value)), token.span)
            }};
        }

        let expr = match &token.kind {
            TokenKind::Null => literal!(Null),
            TokenKind::String(v) => literal!(String(v.clone())),
            TokenKind::InterpolatedString(v) => self.parse_interpolated_string(v)?,
            TokenKind::Path(v) => literal!(Path(self.resolve_path(v.clone()))),
            TokenKind::InterpolatedPath(v) => self.parse_interpolated_path(v)?,
            TokenKind::Int(v) => literal!(Int(*v)),
            TokenKind::Float(v) => literal!(Float(*v)),
            TokenKind::Bool(v) => literal!(Bool(*v)),
            TokenKind::Identifier(_) => self.parse_ident()?,
            TokenKind::LParen => {
                let lparen_token = token.clone();
                consume!(self, LParen);
                let inner_expr = self.parse()?;
                let rparen_token = consume!(self, RParen);
                let span = merge_spans(lparen_token.span, rparen_token.span);
                Expr::new(ExprKind::Parenthesized(Box::new(inner_expr)), span)
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::UnexpectedToken,
                    self.source.clone(),
                    token.span,
                ));
            }
        };

        Ok(expr)
    }

    pub(crate) fn resolve_path(&self, path: PathBuf) -> PathBuf {
        if let Some(origin) = &self.source.path
            && path.is_relative()
        {
            let base = origin.parent().unwrap_or(origin.as_path());
            base.join(path)
        } else {
            path
        }
    }
}
