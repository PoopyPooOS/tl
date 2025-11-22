use crate::{
    merge_spans,
    parser::{
        ast::{
            ExprResult, advance, consume,
            types::{Error, ErrorKind, Expr, ExprKind},
        },
        lexer::types::{Token, TokenKind},
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

        // Arg
        let arg = match advance!(self) {
            Some(Token {
                kind: TokenKind::Identifier(name),
                ..
            }) => name.clone(),
            _ => {
                return Err(Error::new(
                    ErrorKind::ExpectedToken {
                        expected: "argument".into(),
                        found: None,
                    },
                    self.source.clone(),
                    self.closest_span(),
                ));
            }
        };

        // Body
        consume!(self, Colon);

        let expr = self.parse()?;

        let span = merge_spans(start.span, expr.span);

        Ok(Expr::new(
            ExprKind::FnDecl {
                arg,
                expr: Box::new(expr),
            },
            span,
        ))
    }
}
