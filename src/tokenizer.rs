use crate::utils::handle_string_escapes;

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
            '"' => {
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

                push_token!(String, handle_string_escapes(value));
            }

            // Mult-character tokens
            _ if ch.is_alphanumeric() || ch == '_' => {
                let mut value = String::new();

                while let Some(&ch) = chars.peek()
                    && (ch.is_alphanumeric() || ch == '_' || ch == '.')
                {
                    value.push(ch);
                    chars.next();
                }

                match value.as_str() {
                    // Number / Float
                    _ if value.parse::<u64>().is_ok() => push_token!(Number, value.parse::<u64>().unwrap()),
                    _ if value.parse::<f64>().is_ok() => push_token!(Float, value.parse::<f64>().unwrap()),

                    // Boolean
                    "true" => push_token!(Bool, true),
                    "false" => push_token!(Bool, false),

                    // Keyword
                    "let" => push_token!(Let),
                    "import" => push_token!(Import),

                    // Identifier
                    _ => push_token!(Identifier, value),
                }
            }

            _ => {
                return Err(format!("Unexpected token: {ch}"));
            }
        }
    }

    Ok(tokens)
}
