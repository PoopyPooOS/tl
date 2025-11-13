use super::{
    Context, ExprResult,
    types::{Expr, ExprKind},
};
use crate::{
    merge_spans,
    parser::{
        ast::{
            consume,
            types::{Error, ErrorKind, Literal},
        },
        lexer::types::TokenKind,
    },
};

impl super::Parser {
    pub(super) fn parse_fn_decl(&mut self) -> ExprResult {
        let start = self
            .tokens
            .get(self.pos)
            .ok_or(Error::new(
                ErrorKind::NoTokensLeft,
                self.source.clone(),
                self.closest_span(),
            ))?
            .clone();

        // Args
        consume!(self, LParen);
        let mut args = Vec::new();

        while let Some(next_token) = self.tokens.get(self.pos) {
            if next_token.kind == TokenKind::RParen {
                break;
            }

            let name = match self.tokens.get(self.pos) {
                Some(token) => match &token.kind {
                    TokenKind::Identifier(name) => name.clone(),
                    TokenKind::Comma => {
                        self.pos = self.pos.saturating_add(1);
                        continue;
                    }
                    _ => {
                        return Err(Error::new(
                            ErrorKind::ExpectedToken {
                                expected: "identifier".into(),
                                found: Some(token.kind.clone()),
                            },
                            self.source.clone(),
                            token.span,
                        ));
                    }
                },
                _ => {
                    return Err(Error::new(
                        ErrorKind::ExpectedToken {
                            expected: "identifier".into(),
                            found: None,
                        },
                        self.source.clone(),
                        self.closest_span(),
                    ));
                }
            };

            self.pos = self.pos.saturating_add(1);

            args.push(name);
        }

        consume!(self, RParen);

        // Body
        consume!(self, LBrace);

        if self
            .tokens
            .get(self.pos)
            .is_some_and(|token| token.kind == TokenKind::RBrace)
        {
            let end = consume!(self, RBrace);
            let span = merge_spans(start.span, end.span);

            return Ok(Expr::new(
                ExprKind::FnDecl {
                    args,
                    expr: Box::new(Expr::lit(Literal::Null, span)),
                },
                span,
            ));
        }

        let last_context = self.context.clone();
        self.context = Context::Function;

        let expr = self.parse()?;
        let end = consume!(self, RBrace);
        self.context = last_context;

        Ok(Expr::new(
            ExprKind::FnDecl {
                args,
                expr: Box::new(expr),
            },
            merge_spans(start.span, end.span),
        ))
    }
}
