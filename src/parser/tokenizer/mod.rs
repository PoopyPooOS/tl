use crate::source::Source;
use logger::Location;
use types::{Error, ErrorType, Token, TokenType};

pub mod types;

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
    /// # Errors
    /// This function will return an error if a tokenization error occurs.
    #[allow(clippy::too_many_lines)]
    pub fn tokenize(&mut self) -> Result<Vec<Token>, types::Error> {
        let mut tokens = Vec::new();
        let mut chars = self.source.text.chars().peekable();

        macro_rules! push_token {
            ($token:ident, $len:expr) => {{
                tokens.push(Token::new(TokenType::$token, self.line, self.column..=self.column + $len));
                self.column += $len;
                chars.next();
            }};
        }

        let is_valid_char = |ch: char, dots: bool| {
            if ch == '.' && !dots {
                return false;
            }

            ch.is_alphanumeric() || matches!(ch, '_' | '.' | '=' | '!' | '<' | '>')
        };

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
                ',' => push_token!(Comma, 1),
                ':' => push_token!(Colon, 1),
                '.' => push_token!(Dot, 1),

                // Strings
                '"' => {
                    let start = self.column;
                    let mut closed: bool = false;
                    let mut value = String::new();
                    chars.next();
                    self.column += 1;

                    let mut prev_char: Option<char> = None;
                    while let Some(&ch) = chars.peek() {
                        if ch == '"' {
                            if prev_char == Some('\\') {
                                value.pop();
                                value.push(ch);
                            } else {
                                chars.next();
                                self.column += 1;
                                closed = true;
                                break;
                            }
                        } else {
                            value.push(ch);
                        }

                        prev_char = Some(ch);
                        chars.next();
                        self.column += 1;
                    }

                    if !closed {
                        if let Some(path) = &self.source.path {
                            let location = Location::from_path(path, self.line..=self.line)?;
                            return Err(Error::new(ErrorType::UnclosedString, Some(location)));
                        }

                        let location = Location {
                            path: None,
                            text: self.source.text.clone(),
                            lines: self.line..=self.line,
                            section: Some(start..=self.column),
                        };

                        return Err(Error::new(ErrorType::UnclosedString, Some(location)));
                    }

                    tokens.push(Token::new(
                        TokenType::String(handle_string_escapes(&value)),
                        self.line,
                        start..=self.column,
                    ));
                }

                // Parse numbers and floats
                _ if ch.is_ascii_digit() || ch == '.' => {
                    let mut value = String::new();

                    while let Some(&ch) = chars.peek()
                        && (ch.is_ascii_digit() || ch == '.')
                    {
                        value.push(ch);
                        chars.next();

                        // If character is a bang break and the next character is not a equals.
                        // This is to properly handle the `Not` token.
                        if ch == '!' && chars.peek() != Some(&'=') {
                            break;
                        }
                    }

                    self.column += value.len();
                    match value.as_str() {
                        _ if value.parse::<i64>().is_ok() => {
                            tokens.push(Token::new(
                                TokenType::Int(value.parse::<i64>()?),
                                self.line,
                                self.column.saturating_sub(value.len())..=self.column,
                            ));
                        }
                        _ if value.parse::<f64>().is_ok() => {
                            tokens.push(Token::new(
                                TokenType::Float(value.parse::<f64>()?),
                                self.line,
                                self.column.saturating_sub(value.len())..=self.column,
                            ));
                        }
                        _ => (),
                    }
                }

                // Multi-character tokens (literals, keywords, identifiers, operators)
                _ if is_valid_char(ch, false) => {
                    let mut value = String::new();

                    while let Some(&ch) = chars.peek()
                        && is_valid_char(ch, false)
                    {
                        value.push(ch);
                        chars.next();

                        // If character is a bang break and the next character is not a equals.
                        // This is to properly handle the `Not` token.
                        if ch == '!' && chars.peek() != Some(&'=') {
                            break;
                        }
                    }

                    macro_rules! push_long_token {
                        ($token:ident) => {{
                            tokens.push(Token::new(
                                TokenType::$token,
                                self.line,
                                self.column.saturating_sub(value.len())..=self.column,
                            ));
                        }};
                        ($token:ident($value:expr)) => {{
                            tokens.push(Token::new(
                                TokenType::$token($value),
                                self.line,
                                self.column.saturating_sub(value.len())..=self.column,
                            ));
                        }};
                    }

                    self.column += value.len();
                    match value.as_str() {
                        // Null
                        "null" => push_long_token!(Null),

                        // Boolean
                        "true" => push_long_token!(Bool(true)),
                        "false" => push_long_token!(Bool(false)),

                        // Keywords
                        "struct" => push_long_token!(Struct),
                        "enum" => push_long_token!(Enum),
                        "let" => push_long_token!(Let),

                        // Operators
                        "==" => push_long_token!(Eq),
                        "=" => push_long_token!(Equals),
                        "!=" => push_long_token!(NotEq),
                        "!" => push_long_token!(Not),
                        ">=" => push_long_token!(GtEq),
                        ">" => push_long_token!(Gt),
                        "<=" => push_long_token!(LtEq),
                        "<" => push_long_token!(Lt),

                        // Identifier
                        _ => push_long_token!(Identifier(value.to_string())),
                    }
                }

                _ => {
                    if let Some(path) = &self.source.path {
                        let location = Location::from_path(path.clone(), self.line..=self.line)?;
                        return Err(Error::new(ErrorType::UnexpectedToken(ch), Some(location)));
                    }

                    let location = Location {
                        path: None,
                        text: self.source.text.clone(),
                        lines: self.line..=self.line,
                        section: None,
                    };

                    return Err(Error::new(ErrorType::UnexpectedToken(ch), Some(location)));
                }
            }
        }

        Ok(tokens)
    }
}

fn handle_string_escapes(original: impl Into<String>) -> String {
    let mut original: String = original.into();
    let replacements: &[(&str, &str)] = &[
        (r"\\", "\\"),
        ("\\\"", "\""),
        (r"\'", "'"),
        (r"\n", "\n"),
        (r"\r", "\r"),
        (r"\t", "\t"),
        (r"\0", "\0"),
    ];

    for replacement in replacements {
        original = original.replace(replacement.0, replacement.1);
    }

    original
}
