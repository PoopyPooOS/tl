use crate::{
    merge_spans,
    parser::{
        ast::{
            Context, ExprResult,
            types::{Error, ErrorKind, Expr, ExprKind, Literal},
        },
        lexer::types::TokenKind,
    },
};
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

        // TODO: Maybe there's a better way to tell when to stop parsing here? This doesn't allow for `>` to be used in a binary op for inline type sigs
        if check!(0, t if t.is_binary_operator() && !(*t == TokenKind::Gt && self.context == Context::Type))
        {
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

    fn parse_expr_suffixes(&mut self, mut expr: Expr) -> ExprResult {
        let mut full_span = expr.span;

        loop {
            match peek!(0).map(|t| &t.kind) {
                // Object field access: .identifier
                // TODO: Allow for interpolation here
                Some(TokenKind::Dot) => {
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
                    expr = Expr::new(
                        ExprKind::MemberAccess {
                            base: Box::new(expr),
                            field: field_name,
                        },
                        merge_spans(full_span, field_token.span),
                    );
                    full_span = merge_spans(full_span, field_token.span);
                }

                // Array index access: [expr]
                Some(TokenKind::LBracket) => {
                    change_pos!(1);
                    let index = self.parse()?;
                    let end = consume!("']'", TokenKind::RBracket)?;

                    expr = Expr::new(
                        ExprKind::ArrayIndex {
                            base: Box::new(expr),
                            index: Box::new(index),
                        },
                        merge_spans(full_span, end.span),
                    );

                    full_span = merge_spans(full_span, end.span);
                }

                // Function call: (args...)
                Some(TokenKind::LParen) => {
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

                    expr = Expr::new(
                        ExprKind::Call {
                            base: Box::new(expr),
                            args,
                        },
                        merge_spans(full_span, end.span),
                    );

                    full_span = merge_spans(full_span, end.span);
                }

                _ => break,
            }
        }

        Ok(expr)
    }
}
