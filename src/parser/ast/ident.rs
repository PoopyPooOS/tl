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
use tl_macro::{advance, change_pos, check, consume, peek};

impl super::Parser {
    pub(super) fn parse_ident(&mut self) -> ExprResult {
        let token = consume!("identifier", TokenKind::Identifier(_))?.clone();

        let mut expr = match &token.kind {
            TokenKind::Identifier(_) if check!(0, TokenKind::Colon) => {
                // The current identifier would be the function's argument,
                // so we have to go back so that `parse_fn_decl` can parse the identifier as the function argument.
                change_pos!(-1);
                self.parse_fn_decl()?
            }
            TokenKind::Identifier(name) => Expr::ident(name.clone(), token.span),
            _ => unreachable!(),
        };

        let mut full_span = token.span;

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
                    let index_expr = self.parse()?;
                    let end = consume!("']'", TokenKind::RBracket)?;

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
