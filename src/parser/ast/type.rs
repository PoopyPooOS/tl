use crate::parser::{
    ast::{
        Context,
        types::{Error, ErrorKind, Type},
    },
    lexer::types::TokenKind,
};
use tl_macro::{change_pos, consume, peek_or_err};

impl super::Parser {
    pub(super) fn parse_type(&mut self) -> Result<Type, Error> {
        macro_rules! parse_types {
            (
                $($name:ident => $expr:tt),*
                $(,)?
            ) => {
                match &peek_or_err!(0)?.kind {
                    $(
                        TokenKind::Identifier(v) if v.as_str() == stringify!($name) => {
                            change_pos!(1);
                            #[allow(unused_braces)]
                            $expr
                        }
                    ),*
                    _ => {
                        let last_context = self.context;
                        self.context = Context::Type;
                        let ty = self.parse();
                        self.context = last_context;
                        Ok(Type::Runtime(ty?))
                    },
                }
            };
        }

        parse_types! {
            any => { Ok(Type::Any) },
            nothing => { Ok(Type::Nothing) },
            bool => { Ok(Type::Boolean) },
            int => { Ok(Type::Int) },
            uint => { Ok(Type::UInt) },
            float => { Ok(Type::Float) },
            number => { Ok(Type::Number) },
            string => { Ok(Type::String) },
            path => { Ok(Type::Path) },
            list => {
                consume!("'<'", TokenKind::Lt)?;
                let generic = self.parse_type()?;
                consume!("'>'", TokenKind::Gt)?;
                Ok(Type::List(Box::new(generic)))
            },
            object => {
                consume!("'<'", TokenKind::Lt)?;
                let mut generics = Vec::new();
                loop {
                    let (name, _span) = self.parse_ident_plain()?;
                    consume!("':'", TokenKind::Colon)?;
                    let generic = self.parse_type()?;

                    generics.push((name, generic));

                    let token = peek_or_err!(0)?;
                    match &token.kind {
                        TokenKind::Comma => {
                            change_pos!(1);
                        }
                        TokenKind::Gt => break,
                        _ => {
                            return Err(Error::new(
                                ErrorKind::UnexpectedToken,
                                self.source.clone(),
                                token.span,
                            ));
                        }
                    }
                }
                consume!("'>'", TokenKind::Gt)?;

                Ok(Type::Object(generics.into_boxed_slice()))
            },
            either => {
                consume!("'<'", TokenKind::Lt)?;
                let mut generics = Vec::new();
                loop {
                    let generic = self.parse_type()?;

                    generics.push(generic);

                    let token = peek_or_err!(0)?;
                    match &token.kind {
                        TokenKind::Comma => (),
                        TokenKind::Gt => break,
                        _ => {
                            return Err(Error::new(
                                ErrorKind::ExpectedToken {
                                    expected: "',' to continue generic list or '>' to end it"
                                        .to_owned(),
                                    found: Some(token.kind.clone()),
                                },
                                self.source.clone(),
                                token.span,
                            ));
                        }
                    }
                }
                consume!("'>'", TokenKind::Gt)?;

                Ok(Type::Either(generics.into_boxed_slice()))
            },
            function => { Ok(Type::Function) },
            thunk => {
                consume!("'<'", TokenKind::Lt)?;
                let generic = self.parse_type()?;
                consume!("'>'", TokenKind::Gt)?;
                Ok(Type::Thunk(Box::new(generic)))
            },
        }
    }
}
