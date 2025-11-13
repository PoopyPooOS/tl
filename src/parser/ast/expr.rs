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
            TokenKind::LParen => {
                // Function Declaration
                if let Some(next_token) = self.tokens.get(self.pos.saturating_add(1))
                    && matches!(
                        next_token.kind,
                        TokenKind::Identifier(_) | TokenKind::RParen
                    )
                {
                    return self.parse_fn_decl();
                }

                None
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

        self.parse_literal()
    }

    pub(super) fn parse_literal(&mut self) -> ExprResult {
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
            TokenKind::Path(v) => literal!(Path(v.clone())),
            TokenKind::InterpolatedPath(v) => self.parse_interpolated_path(v)?,
            TokenKind::Int(v) => literal!(Int(*v)),
            TokenKind::Float(v) => literal!(Float(*v)),
            TokenKind::Bool(v) => literal!(Bool(*v)),
            TokenKind::Identifier(_) => self.parse_ident()?,
            _ => {
                return Err(Error::new(
                    ErrorKind::UnexpectedToken,
                    self.source.clone(),
                    token.span,
                ));
            }
        };

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
}
