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
    Star,
    Slash,

    // Parenthesis
    LParen,
    RParen,

    // Misc
    Equals,
}

#[allow(clippy::unnecessary_wraps)]
pub fn tokenize(input: impl Into<String>) -> Result<Vec<Token>, String> {
    let input: String = input.into();
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    macro_rules! token {
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

            // Parenthesis
            '(' => {
                token!(LParen);
                chars.next();
            }
            ')' => {
                token!(RParen);
                chars.next();
            }

            // Operators
            '+' => {
                token!(Plus);
                chars.next();
            }
            '-' => {
                token!(Minus);
                chars.next();
            }
            '*' => {
                token!(Star);
                chars.next();
            }
            '/' => {
                token!(Slash);
                chars.next();
            }
            '=' => {
                token!(Equals);
                chars.next();
            }

            // Mult-character tokens
            _ if ch.is_alphanumeric() => {
                let mut value = String::new();

                while let Some(&ch) = chars.peek()
                    && (ch.is_alphanumeric() || ch == '_')
                {
                    value.push(ch);
                    chars.next();
                }

                dbg!(&value);

                match value.as_str() {
                    // Number / Float
                    _ if value.parse::<u64>().is_ok() => token!(Number, value.parse::<u64>().unwrap()),
                    _ if value.parse::<f64>().is_ok() => token!(Float, value.parse::<f64>().unwrap()),

                    // String
                    _ if value.starts_with('"') && value.ends_with('"') => {
                        token!(String, handle_string_escapes(value[1..value.len() - 1].to_string()));
                    }

                    // Boolean
                    "true" => token!(Bool, true),
                    "false" => token!(Bool, false),

                    // Keyword
                    "let" => token!(Let),
                    "import" => token!(Import),

                    // Identifier
                    _ => token!(Identifier, value),
                }
            }

            _ => {
                return Err(format!("Unexpected token: {ch}"));
            }
        }
        chars.next();
    }

    Ok(tokens)
}
