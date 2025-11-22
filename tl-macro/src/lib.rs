use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Expr, ExprLit, ExprUnary, Lit, Pat, Result, Token, UnOp,
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::If,
};

#[proc_macro]
pub fn change_pos(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    match &expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit_int),
            ..
        }) => {
            let s = lit_int.to_string();
            if let Some(stripped) = s.strip_prefix('-') {
                let value: usize = stripped.parse().unwrap();

                quote! {
                    self.pos = self.pos.saturating_sub(#value);
                }
            } else {
                let value: usize = s.parse().unwrap();

                quote! {
                    self.pos = self.pos.saturating_add(#value);
                }
            }
        }
        _ => quote! {
            let amount: isize = #expr as isize;

            if amount.is_negative() {
                self.pos = self.pos.saturating_sub(amount.unsigned_abs());
            } else {
                self.pos = self.pos.saturating_add(amount.unsigned_abs());
            }
        },
    }
    .into()
}

/// Peek at `self.pos +- {index}`
#[proc_macro]
pub fn peek(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    match &expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit_int),
            ..
        }) => {
            let value: usize = lit_int.base10_digits().parse().unwrap();

            quote! {
                self.tokens.get(self.pos.saturating_add(#value))
            }
        }
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) => {
            if let Expr::Lit(ExprLit {
                lit: Lit::Int(ref lit_int),
                ..
            }) = **expr
            {
                let value: usize = lit_int.base10_digits().parse().unwrap();

                quote! {
                    self.tokens.get(self.pos.saturating_sub(#value))
                }
            } else {
                quote! {
                    compile_error!("expected integer literal in unary negation")
                }
            }
        }
        _ => quote! {{
            let index: isize = #expr;

            self.tokens.get(if index.is_negative() {
                self.pos.saturating_sub(index.unsigned_abs())
            } else {
                self.pos.saturating_add(index.unsigned_abs())
            })
        }},
    }
    .into()
}

struct PeekOrErrArgs {
    index: Expr,
    span: Option<Expr>,
}

impl Parse for PeekOrErrArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let index = input.parse()?;
        let span = if input.parse::<Token![,]>().is_ok() {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(PeekOrErrArgs { index, span })
    }
}

/// Peek at `self.pos +- {index}` but return `NoTokensLeft` error if there's no token found at the index.
#[proc_macro]
pub fn peek_or_err(input: TokenStream) -> TokenStream {
    let PeekOrErrArgs { index, span } = parse_macro_input!(input as PeekOrErrArgs);

    let span = span
        .map(ToTokens::into_token_stream)
        .unwrap_or(quote! { self.closest_span() });

    quote! {
        tl_macro::peek!(#index).ok_or(Error::new(
            ErrorKind::NoTokensLeft,
            self.source.clone(),
            #span,
        ))
    }
    .into()
}

struct GuardPat {
    pat: Pat,
    guard: Option<(If, Box<Expr>)>,
}

impl Parse for GuardPat {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(GuardPat {
            pat: Pat::parse_multi_with_leading_vert(input)?,
            guard: {
                if input.peek(Token![if]) {
                    let if_token: Token![if] = input.parse()?;
                    let guard: Expr = input.parse()?;
                    Some((if_token, Box::new(guard)))
                } else {
                    None
                }
            },
        })
    }
}

impl ToTokens for GuardPat {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.pat.to_tokens(tokens);
        if let Some((if_token, guard)) = &self.guard {
            if_token.to_tokens(tokens);
            guard.to_tokens(tokens);
        }
    }
}

struct CheckArgs {
    index: Expr,
    check: GuardPat,
}

impl Parse for CheckArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let index = input.parse()?;
        input.parse::<Token![,]>()?;
        let check = input.parse()?;

        Ok(CheckArgs { index, check })
    }
}

/// Peek at `self.pos +- {index}` and then check if the token is the right one with your given function.
#[proc_macro]
pub fn check(input: TokenStream) -> TokenStream {
    let CheckArgs { index, check } = parse_macro_input!(input as CheckArgs);

    quote! {{
        if let Some(token) = tl_macro::peek!(#index) {
            matches!(&token.kind, #check)
        } else {
            false
        }
    }}
    .into()
}

#[proc_macro]
pub fn advance(_: TokenStream) -> TokenStream {
    quote! {{
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos = self.pos.saturating_add(1);
        }
        token
    }}
    .into()
}

struct ConsumeArgs {
    expected: Expr,
    pattern: Expr,
}

impl Parse for ConsumeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let expected = input.parse()?;
        input.parse::<Token![,]>()?;
        let pattern = input.parse()?;

        Ok(ConsumeArgs { expected, pattern })
    }
}

#[proc_macro]
pub fn consume(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as ConsumeArgs);

    let ConsumeArgs { expected, pattern } = args;

    quote! {{
        use crate::parser::ast::types::{Error, ErrorKind};

        match tl_macro::advance!() {
            Some(token) => {
                if matches!(&token.kind, #pattern) {
                    Ok(token)
                } else {
                    Err(Error::new(
                        ErrorKind::ExpectedToken {
                            expected: #expected.into(),
                            found: Some(token.kind.clone()),
                        },
                        self.source.clone(),
                        token.span,
                    ))
                }
            }
            None => Err(Error::new(
                ErrorKind::ExpectedToken {
                    expected: #expected.into(),
                    found: None,
                },
                self.source.clone(),
                self.closest_span(),
            )),
        }
    }}
    .into()
}
