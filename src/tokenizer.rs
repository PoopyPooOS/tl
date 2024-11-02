use crate::utils::handle_string_escapes;
use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Token {
    // Literals
    String(String),
    Number(u64),
    Float(f64),
    Bool(bool),

    // Identifiers
    Identifier(String),

    // Keywords
    Let,
    Import,

    // Operators
    Plus,
    Minus,
    Multiply,
    Slash,

    // Parenthesis
    LParen,
    RParen,

    // Misc
    Equals,
}

#[allow(clippy::too_many_lines)] // mind your own bussiness clippy
pub fn tokenize(input: impl Into<String>) -> Result<Vec<Token>, String> {
    let input: String = input.into();
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    macro_rules! push_token {
        ($token_type:ident) => {
            tokens.push(Token::$token_type)
        };
        ($token_type:ident, $value:expr) => {
            tokens.push(Token::$token_type($value))
        };
    }

    while let Some(&ch) = chars.peek() {
        match ch {
            // Whitespace
            ' ' | '\t' | '\n' => {
                chars.next();
                continue;
            }

            // Comments / Slash operator
            '/' => {
                if let Some(next_ch) = chars.clone().nth(1) {
                    if next_ch == '/' {
                        chars.next();
                        chars.next();

                        while let Some(&ch) = chars.peek() {
                            if ch == '\n' {
                                break;
                            }
                            chars.next();
                        }
                        continue;
                    }
                }

                push_token!(Slash);
                chars.next();
            }

            // Parenthesis
            '(' => {
                push_token!(LParen);
                chars.next();
            }
            ')' => {
                push_token!(RParen);
                chars.next();
            }

            // Operators
            '+' => {
                push_token!(Plus);
                chars.next();
            }
            '-' => {
                push_token!(Minus);
                chars.next();
            }
            '*' => {
                push_token!(Multiply);
                chars.next();
            }
            '=' => {
                push_token!(Equals);
                chars.next();
            }

            // Strings
            '"' => tokens.push(tokenize_string(&mut chars)?),

            // Mult-character tokens (literals, keywords, identifiers)
            _ if ch.is_alphanumeric() || ch == '_' => tokens.extend(tokenize_multi_char(&mut chars)),

            _ => {
                return Err(format!("Unexpected token: {ch}"));
            }
        }
    }

    Ok(tokens)
}

pub fn tokenize_string(chars: &mut Peekable<Chars<'_>>) -> Result<Token, String> {
    let mut closed: bool = false;
    let mut value = String::new();
    chars.next();

    while let Some(&ch) = chars.peek() {
        if ch == '"' {
            chars.next();
            closed = true;
            break;
        }

        value.push(ch);
        chars.next();
    }

    if !closed {
        return Err("Unclosed string literal".to_string());
    }

    Ok(Token::String(handle_string_escapes(value)))
}

pub fn tokenize_multi_char(chars: &mut Peekable<Chars<'_>>) -> Vec<Token> {
    let mut value = String::new();
    let mut tokens = Vec::new();

    while let Some(&ch) = chars.peek()
        && (ch.is_alphanumeric() || ch == '_' || ch == '.')
    {
        value.push(ch);
        chars.next();
    }

    match value.as_str() {
        // Number / Float
        _ if value.parse::<u64>().is_ok() => tokens.push(Token::Number(value.parse::<u64>().unwrap())),
        _ if value.parse::<f64>().is_ok() => tokens.push(Token::Float(value.parse::<f64>().unwrap())),

        // Boolean
        "true" => tokens.push(Token::Bool(true)),
        "false" => tokens.push(Token::Bool(false)),

        // Keywords
        "let" => tokens.push(Token::Let),
        "import" => tokens.push(Token::Import),

        // Identifier
        _ => tokens.push(Token::Identifier(value)),
    }

    tokens
}
