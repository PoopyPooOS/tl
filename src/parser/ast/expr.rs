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
use miette::SourceSpan;
use std::path::PathBuf;
use tl_macro::{advance, change_pos, check, consume, peek, peek_or_err};

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
            TokenKind::With => Some(self.parse_with()?),
            _ => None,
        };

        if let Some(expr) = expr {
            return Ok(expr);
        }

        let expr = self.parse_primary()?;
        let expr = self.parse_expr_suffixes(expr)?;

        if check!(0, TokenKind::Pipe) {
            return self.parse_piped_expr(expr);
        }

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
            TokenKind::Pipe
                if check!(1, TokenKind::Identifier(_))
                    && check!(2, TokenKind::Comma | TokenKind::Colon | TokenKind::Pipe) =>
            {
                self.parse_fn_decl()?
            }
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

    /// Parses member access (dot notation): `.identifier`
    /// Assumes the dot has not been consumed yet.
    pub(super) fn parse_member_access(
        &mut self,
        base: Expr,
        current_span: SourceSpan,
    ) -> ExprResult {
        // Consume dot
        change_pos!(1);
        let field_token = advance!().ok_or({
            Error::new(
                ErrorKind::ExpectedIdentifierAfterDot,
                self.source.clone(),
                self.closest_span(),
            )
        })?;

        let field_name = match &field_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                return Err(Error::new(
                    ErrorKind::ExpectedToken {
                        expected: "identifier".into(),
                        found: None,
                    },
                    self.source.clone(),
                    field_token.span,
                ));
            }
        };

        Ok(Expr::new(
            ExprKind::MemberAccess {
                base: Box::new(base),
                field: field_name,
            },
            merge_spans(current_span, field_token.span),
        ))
    }

    /// Parses array index access: `[expr]`
    /// Assumes the left bracket has not been consumed yet.
    pub(super) fn parse_array_index(&mut self, base: Expr, current_span: SourceSpan) -> ExprResult {
        change_pos!(1);
        let index = self.parse()?;
        let end = consume!("']'", TokenKind::RBracket)?;

        Ok(Expr::new(
            ExprKind::ArrayIndex {
                base: Box::new(base),
                index: Box::new(index),
            },
            merge_spans(current_span, end.span),
        ))
    }

    /// Parses function call: `(args...)`
    /// Assumes the left parenthesis has not been consumed yet.
    pub(super) fn parse_call(&mut self, base: Expr, current_span: SourceSpan) -> ExprResult {
        change_pos!(1);
        let mut args = Vec::new();

        while let Some(token) = peek!(0)
            && token.kind != TokenKind::RParen
        {
            if token.kind == TokenKind::Comma {
                change_pos!(1);
                continue;
            }

            args.push(self.parse()?);
        }

        let end = consume!("')'", TokenKind::RParen)?;

        Ok(Expr::new(
            ExprKind::Call {
                base: Box::new(base),
                args,
            },
            merge_spans(current_span, end.span),
        ))
    }

    pub(super) fn parse_expr_suffixes(&mut self, mut expr: Expr) -> ExprResult {
        let mut full_span = expr.span;

        loop {
            match peek!(0).map(|t| &t.kind) {
                Some(TokenKind::Dot) => {
                    expr = self.parse_member_access(expr, full_span)?;
                    full_span = expr.span;
                }
                Some(TokenKind::LBracket) => {
                    expr = self.parse_array_index(expr, full_span)?;
                    full_span = expr.span;
                }
                Some(TokenKind::LParen) => {
                    expr = self.parse_call(expr, full_span)?;
                    full_span = expr.span;
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_piped_expr(&mut self, input: Expr) -> ExprResult {
        consume!("'|'", TokenKind::Pipe)?;

        let right_expr = self.parse_primary()?;
        let right_expr = self.parse_expr_suffixes(right_expr)?;

        let (base, args) = match &right_expr.kind {
            ExprKind::Call { base, args } => (base.clone(), args.clone()),
            _ => (Box::new(right_expr.clone()), vec![]),
        };

        let span = merge_spans(input.span, right_expr.span);

        Ok(Expr::new(
            ExprKind::PipedCall {
                input: Box::new(input),
                base,
                args,
            },
            span,
        ))
    }
}
