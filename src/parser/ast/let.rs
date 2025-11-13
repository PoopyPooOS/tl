use crate::{
    merge_spans,
    parser::{
        ast::{
            ExprResult, advance, consume,
            types::{Error, ErrorKind, Expr, ExprKind},
        },
        lexer::types::TokenKind,
    },
};

impl super::Parser {
    pub(super) fn parse_let(&mut self) -> ExprResult {
        let start = self
            .tokens
            .get(self.pos)
            .ok_or(Error::new(
                ErrorKind::NoTokensLeft,
                self.source.clone(),
                self.closest_span(),
            ))?
            .clone();

        consume!(self, Let);

        let mut bindings = Vec::new();

        loop {
            let token = self
                .tokens
                .get(self.pos)
                .ok_or(Error::new(
                    ErrorKind::NoTokensLeft,
                    self.source.clone(),
                    self.closest_span(),
                ))?
                .clone();

            if token.kind == TokenKind::In {
                break;
            }

            let name_token = advance!(self).ok_or(Error::new(
                ErrorKind::NoTokensLeft,
                self.source.clone(),
                token.span,
            ))?;

            let name = if let TokenKind::Identifier(name) = &name_token.kind {
                name.clone()
            } else {
                return Err(Error::new(
                    ErrorKind::ExpectedToken {
                        expected: "identifier".into(),
                        found: None,
                    },
                    self.source.clone(),
                    token.span,
                ));
            };

            consume!(self, Equals);

            let value = self.parse()?;
            bindings.push((name, value));
        }

        consume!(self, In);

        let body = self.parse()?;
        let end_span = body.span;

        Ok(Expr::new(
            ExprKind::LetIn {
                bindings,
                expr: Box::new(body),
            },
            merge_spans(start.span, end_span),
        ))
    }
}
