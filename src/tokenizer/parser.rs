use super::types::{Token, TokenType};
use crate::utils::handle_string_escapes;
use logger::{error, make_error, Location, Log};
use std::{fs, path::PathBuf, process};

pub struct Parser {
    pub path: PathBuf,
    pub input: String,
    pub line: usize,
    pub column: usize,
}

impl Parser {
    pub fn new(path: impl Into<String>) -> Self {
        let path: String = path.into();
        let input = match fs::read_to_string(&path) {
            Ok(input) => input,
            Err(err) => {
                error!(format!("Failed to read from file '{}':\n{err:#?}", &path));
                process::exit(1);
            }
        };

        println!("Generating AST from:\n```\n{input}\n```");

        Self {
            path: path.into(),
            input,
            line: 0,
            column: 0,
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn tokenize(&mut self) -> Result<Vec<Token>, Log> {
        let mut tokens = Vec::new();
        let mut chars = self.input.chars().peekable();

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
                            self.column += 1;
                            chars.next();
                            self.column += 1;

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

                        push_token!(Slash, 1);
                    }
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
                '=' => push_token!(Equals, 1),

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
                        let location =
                            Location::new_with_section(&self.path, self.line..=self.line, self.column - value.len()..=self.column);
                        return Err(make_error!("Unclosed string literal", location: location));
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
                        "import" => tokens.push(Token::new(TokenType::Import, self.line, self.column - value.len(), value.len())),

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
                    return Err(make_error!(format!("Unexpected token: {ch}"), location: Location::new(&self.path, self.line..=self.line)))
                }
            }
        }

        Ok(tokens)
    }
}

// pub fn tokenize(input: impl Into<String>) -> Result<Vec<Token>, String> {
//     let input: String = input.into();
//     let mut tokens = Vec::new();
//     let mut chars = input.chars().peekable();

//     while let Some(&ch) = chars.peek() {
//         match ch {
//             // Whitespace
//             ' ' | '\t' | '\n' => {
//                 chars.next();
//                 continue;
//             }

//             // Comments / Slash operator
//             '/' => {
//                 if let Some(next_ch) = chars.clone().nth(1) {
//                     if next_ch == '/' {
//                         chars.next();
//                         chars.next();

//                         while let Some(&ch) = chars.peek() {
//                             if ch == '\n' {
//                                 break;
//                             }
//                             chars.next();
//                         }
//                         continue;
//                     }
//                 }

//                 push_token!(Slash);
//             }

//             // Do not try to simplify the match arm body, the push_token macro wont work if you do so.

//             // Brackets
//             '(' => {
//                 push_token!(LParen);
//             }
//             ')' => {
//                 push_token!(RParen);
//             }
//             '[' => {
//                 push_token!(LBracket);
//             }
//             ']' => {
//                 push_token!(RBracket);
//             }
//             '{' => {
//                 push_token!(LBrace);
//             }
//             '}' => {
//                 push_token!(RBrace);
//             }

//             // Operators
//             '+' => {
//                 push_token!(Plus);
//             }
//             '-' => {
//                 push_token!(Minus);
//             }
//             '*' => {
//                 push_token!(Multiply);
//             }
//             '=' => {
//                 push_token!(Equals);
//             }

//             // Strings
//             '"' => tokens.push(tokenize_string(&mut chars)?),

//             // Multi-character tokens (literals, keywords, identifiers)
//             _ if ch.is_alphanumeric() || ch == '_' => tokens.extend(tokenize_multi_char(&mut chars)),

//             _ => return Err(format!("Unexpected token: {ch}")),
//         }
//     }

//     Ok(tokens)
// }
