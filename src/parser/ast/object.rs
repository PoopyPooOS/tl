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
use indexmap::IndexMap;
use tl_macro::{advance, change_pos, check, consume, peek, peek_or_err};

impl super::Parser {
    pub(super) fn parse_object(&mut self) -> ExprResult {
        let start = consume!("'{'", TokenKind::LBrace)?.clone();

        let last_context = self.context.clone();
        self.context = Context::Object;

        let mut fields = IndexMap::new();

        loop {
            let token = peek!(0)
                .ok_or(Error::new(
                    ErrorKind::ExpectedToken {
                        expected: "'}'".to_owned(),
                        found: None,
                    },
                    self.source.clone(),
                    self.closest_span(),
                ))?
                .clone();

            if token.kind == TokenKind::RBrace {
                change_pos!(1);
                break;
            }

            let mut key_parts = Vec::new();
            loop {
                let token = advance!().ok_or(Error::new(
                    ErrorKind::NoTokensLeft,
                    self.source.clone(),
                    token.span,
                ))?;

                match &token.kind {
                    TokenKind::Identifier(name) | TokenKind::String(name) => {
                        key_parts.push(name.clone());
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
                }

                if check!(0, TokenKind::Dot) {
                    advance!();
                    continue;
                }

                break;
            }

            match advance!() {
                Some(token) => match token.kind {
                    TokenKind::Equals => (),
                    TokenKind::Colon => {
                        return Err(Error::new(
                            ErrorKind::UnexpectedColonInObjectKV,
                            self.source.clone(),
                            token.span,
                        ));
                    }
                    _ => {
                        return Err(Error::new(
                            ErrorKind::ExpectedSeparatorInObjectKV,
                            self.source.clone(),
                            token.span,
                        ));
                    }
                },
                _ => {
                    return Err(Error::new(
                        ErrorKind::ExpectedSeparatorInObjectKV,
                        self.source.clone(),
                        self.closest_span(),
                    ));
                }
            }

            let value = self.parse()?;
            let nested = Self::nest_object(key_parts, value);

            Self::merge_object(&mut fields, nested);
        }

        self.context = last_context;
        let end = peek_or_err!(-1)?;

        Ok(Expr::new(
            ExprKind::Literal(Literal::Object(fields)),
            merge_spans(start.span, end.span),
        ))
    }

    fn nest_object(mut parts: Vec<String>, value: Expr) -> Expr {
        #[allow(clippy::unwrap_used)]
        let last = parts.pop().unwrap();

        let mut inner = IndexMap::new();
        inner.insert(last, value.clone());

        let mut expr = Expr::new(ExprKind::Literal(Literal::Object(inner)), value.span);

        while let Some(part) = parts.pop() {
            let mut outer = IndexMap::new();
            outer.insert(part, expr.clone());
            expr = Expr::new(ExprKind::Literal(Literal::Object(outer)), expr.span);
        }

        expr
    }

    fn merge_object(target: &mut IndexMap<String, Expr>, nested: Expr) {
        if let ExprKind::Literal(Literal::Object(new_map)) = nested.kind {
            for (k, v) in new_map {
                if let Some(existing) = target.get_mut(&k)
                    && let (
                        ExprKind::Literal(Literal::Object(existing_map)),
                        ExprKind::Literal(Literal::Object(new_sub)),
                    ) = (&mut existing.kind, v.kind.clone())
                {
                    for (nk, nv) in new_sub {
                        Self::merge_object(existing_map, Self::nest_object(vec![nk], nv));
                    }
                    continue;
                }
                target.insert(k, v);
            }
        }
    }
}
