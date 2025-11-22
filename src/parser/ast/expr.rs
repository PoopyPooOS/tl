use crate::{
    merge_spans,
    parser::{
        ast::{
            ExprResult,
            types::{Error, ErrorKind, Expr, ExprKind, Literal},
        },
        lexer::types::TokenKind,
    },
};
use std::path::PathBuf;
use tl_macro::{check, consume, peek_or_err};

impl super::Parser {
    /// Generates an AST based on the tokens of this [`Parser`].
    /// # Errors
    /// This function will return an error if a AST generation error occurs.
    pub fn parse(&mut self) -> ExprResult {
        let token = peek_or_err!(0)?;

        let expr = match token.kind {
            TokenKind::Not => {
                let not_span = consume!("'!' (Not)", TokenKind::Not)?.span;
                let expr = self.parse()?;

                let span = merge_spans(not_span, expr.span);

                Some(Expr::new(ExprKind::Not(Box::new(expr)), span))
            }
            TokenKind::Let => Some(self.parse_let()?),
            _ => None,
        };

        if let Some(expr) = expr {
            return Ok(expr);
        }

        let expr = self.parse_primary()?;

        if check!(0, t if t.is_binary_operator()) {
            return self.parse_binary_op_with_left(0, expr);
        }

        Ok(expr)
    }

    pub(super) fn parse_primary(&mut self) -> ExprResult {
        let token = peek_or_err!(0)?.clone();

        macro_rules! literal {
            ($variant:ident) => {{
                tl_macro::change_pos!(1);
                Expr::new(ExprKind::Literal(Literal::$variant), token.span)
            }};
            ($variant:ident($value:expr)) => {{
                tl_macro::change_pos!(1);
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
            TokenKind::LBrace => self.parse_object()?,
            TokenKind::LBracket => self.parse_array()?,
            TokenKind::LParen => {
                let lparen_span = consume!("'('", TokenKind::LParen)?.span;
                let inner_expr = self.parse()?;
                let rparen_span = consume!("'('", TokenKind::RParen)?.span;

                let span = merge_spans(lparen_span, rparen_span);

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
