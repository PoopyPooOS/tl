use super::types::{Token, TokenType};
use crate::{source::Source, utils::handle_string_escapes};
use logger::{make_error, Location, Log};

pub struct Parser<'a> {
    pub(crate) source: &'a Source,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: impl Into<&'a Source>) -> Self {
        Self {
            source: source.into(),
            line: 0,
            column: 0,
        }
    }

    /// Tokenizes the source code inside the [`Parser`] struct.
    ///
    /// # Panics
    /// Panics if a number cannot be parsed.
    ///
    /// # Errors
    /// This function will return an error if a tokenization error occurs.
    #[allow(clippy::too_many_lines)]
    pub fn tokenize(&mut self) -> Result<Vec<Token>, Box<Log>> {
        let mut tokens = Vec::new();
        let mut chars = self.source.text.chars().peekable();

        macro_rules! push_token {
            ($token:ident, $len:expr) => {{
                tokens.push(Token::new(TokenType::$token, self.line, self.column, $len));
                self.column += 1;
                chars.next();
            }};
        }

        while let Some(&ch) = chars.peek() {
            match ch {
                // Whitespace
                ' ' | '\t' => {
                    chars.next();
                    self.column += 1;
                    continue;
                }
                '\n' => {
                    chars.next();
                    self.line += 1;
                    self.column = 0;
                    continue;
                }

                // Comments / Slash operator
                '/' => {
                    if let Some(next_ch) = chars.clone().nth(1) {
                        if next_ch == '/' {
                            chars.next();
                            chars.next();
                            self.column += 2;

                            while let Some(&ch) = chars.peek() {
                                if ch == '\n' {
                                    self.line += 1;
                                    break;
                                }
                                chars.next();
                                self.column += 1;
                            }
                            continue;
                        }
                    }

                    push_token!(Slash, 1);
                }

                // Brackets
                '(' => push_token!(LParen, 1),
                ')' => push_token!(RParen, 1),
                '[' => push_token!(LBracket, 1),
                ']' => push_token!(RBracket, 1),
                '{' => push_token!(LBrace, 1),
                '}' => push_token!(RBrace, 1),

                // Binary operators
                '+' => push_token!(Plus, 1),
                '-' => push_token!(Minus, 1),
                '*' => push_token!(Multiply, 1),

                // Misc
                '=' => push_token!(Equals, 1),
                ',' => push_token!(Comma, 1),
                ':' => push_token!(Colon, 1),

                // Strings
                '"' => {
                    let mut closed: bool = false;
                    let mut value = String::new();
                    chars.next();
                    self.column += 1;

                    while let Some(&ch) = chars.peek() {
                        if ch == '"' {
                            chars.next();
                            self.column += 1;
                            closed = true;
                            break;
                        }

                        value.push(ch);
                        chars.next();
                        self.column += 1;
                    }

                    if !closed {
                        if let Some(path) = &self.source.path {
                            let location =
                                Location::new_with_section(path.clone(), self.line..=self.line, self.column - value.len()..=self.column);
                            return Err(Box::new(make_error!("Unclosed string literal", location: location)));
                        }

                        let location = Location {
                            path: None,
                            text: Some(self.source.text.clone()),
                            lines: self.line..=self.line,
                            section: Some(self.column - value.len()..=self.column),
                        };

                        return Err(Box::new(make_error!("Unclosed string literal", location: location)));
                    }

                    tokens.push(Token::new(
                        TokenType::String(handle_string_escapes(&value)),
                        self.line,
                        self.column - value.len() - 2,
                        value.len() + 2,
                    ));
                }

                // Multi-character tokens (literals, keywords, identifiers)
                _ if ch.is_alphanumeric() || ch == '_' => {
                    let mut value = String::new();

                    while let Some(&ch) = chars.peek()
                        && (ch.is_alphanumeric() || ch == '_' || ch == '.')
                    {
                        value.push(ch);
                        chars.next();
                    }

                    self.column += value.len();
                    match value.as_str() {
                        // Number / Float
                        _ if value.parse::<u64>().is_ok() => tokens.push(Token::new(
                            TokenType::Number(value.parse::<i64>().unwrap()),
                            self.line,
                            self.column - value.len(),
                            value.len(),
                        )),
                        _ if value.parse::<f64>().is_ok() => tokens.push(Token::new(
                            TokenType::Float(value.parse::<f64>().unwrap()),
                            self.line,
                            self.column - value.len() + 1, // account for the .
                            value.len(),
                        )),

                        // Boolean
                        "true" => tokens.push(Token::new(TokenType::Bool(true), self.line, self.column - value.len(), value.len())),
                        "false" => tokens.push(Token::new(
                            TokenType::Bool(false),
                            self.line,
                            self.column - value.len(),
                            value.len(),
                        )),

                        // Keywords
                        "let" => tokens.push(Token::new(TokenType::Let, self.line, self.column - value.len(), value.len())),
                        "fn" => tokens.push(Token::new(TokenType::Fn, self.line, self.column - value.len(), value.len())),

                        // Identifier
                        _ => tokens.push(Token::new(
                            TokenType::Identifier(value.clone()),
                            self.line,
                            self.column - value.len(),
                            value.len(),
                        )),
                    }
                }

                _ => {
                    if let Some(path) = &self.source.path {
                        let location = Location::new(path.clone(), self.line..=self.line);
                        return Err(Box::new(make_error!(format!("Unexpected token: '{ch}'"), location: location)));
                    }

                    let location = Location {
                        path: None,
                        text: Some(self.source.text.clone()),
                        lines: self.line..=self.line,
                        section: None,
                    };

                    return Err(Box::new(make_error!(format!("Unexpected token: '{ch}'"), location: location)));
                }
            }
        }

        Ok(tokens)
    }
}
