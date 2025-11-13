use super::{
    ExprResult,
    types::{Expr, ExprKind},
};
use crate::{
    merge_spans,
    parser::{
        ast::{
            advance, consume,
            types::{Error, ErrorKind, Literal},
        },
        lexer::types::TokenKind,
    },
};

impl super::Parser {
    pub(super) fn parse_ident(&mut self) -> ExprResult {
        let token = advance!(self).ok_or(Error::new(
            ErrorKind::NoTokensLeft,
            self.source.clone(),
            self.closest_span(),
        ))?;

        let mut expr = match &token.kind {
            TokenKind::Identifier(name) => Expr::ident(name.clone(), token.span),
            _ => unreachable!(),
        };
        let mut full_span = token.span;

        loop {
            match self.tokens.get(self.pos).map(|t| &t.kind) {
                // Object field access: .identifier
                // TODO: Allow for interpolation here
                Some(TokenKind::Dot) => {
                    self.pos = self.pos.saturating_add(1);
                    let field_token = advance!(self).ok_or({
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
                        ExprKind::ObjectAccess {
                            base: Box::new(expr),
                            field: field_name,
                        },
                        merge_spans(full_span, field_token.span),
                    );
                    full_span = merge_spans(full_span, field_token.span);
                }

                // Array index access: [expr]
                Some(TokenKind::LBracket) => {
                    self.pos = self.pos.saturating_add(1);
                    let index_expr = self.parse()?;
                    let end = consume!(self, RBracket);

                    expr = match index_expr.kind {
                        ExprKind::Literal(Literal::Int(v)) if v >= 0 => Expr::new(
                            ExprKind::ArrayIndex {
                                base: Box::new(expr),
                                index: v as usize,
                            },
                            merge_spans(full_span, end.span),
                        ),
                        _ => Expr::new(
                            ExprKind::ArrayIndex {
                                base: Box::new(expr),
                                index: 0,
                            },
                            merge_spans(full_span, end.span),
                        ),
                    };

                    full_span = merge_spans(full_span, end.span);
                }

                // Function call: (args...)
                Some(TokenKind::LParen) => {
                    self.pos = self.pos.saturating_add(1);
                    let mut args = Vec::new();
                    while let Some(token) = self.tokens.get(self.pos)
                        && token.kind != TokenKind::RParen
                    {
                        if token.kind == TokenKind::Comma {
                            self.pos = self.pos.saturating_add(1);
                            continue;
                        }

                        args.push(self.parse()?);
                    }
                    let end = consume!(self, RParen);

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
