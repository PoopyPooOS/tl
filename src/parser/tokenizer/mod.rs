use std::{iter::Peekable, path::PathBuf, str::Chars};

use crate::source::Source;
use logger::{location::Section, Location};
use types::{Error, ErrorType, Token, TokenType};

pub mod types;

pub struct Parser {
    pub(crate) source: Source,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl Parser {
    pub fn new(source: impl Into<Source>) -> Self {
        if cfg!(debug_assertions) {
            logger::set_app_name!("Tokenizer");
        }

        Self {
            source: source.into(),
            line: 0,
            column: 0,
        }
    }

    /// Tokenizes the source code inside the [`Parser`] struct.
    /// # Errors
    /// This function will return an error if a tokenization error occurs.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, types::Error> {
        let mut tokens = Vec::new();
        let mut chars = self.source.text.chars().peekable();

        macro_rules! push_token {
            ($token:ident, $len:expr) => {{
                tokens.push(Token::new(
                    TokenType::$token,
                    Section::new(
                        self.line..=self.line,
                        self.column..=self.column.saturating_add($len),
                    ),
                ));
                self.column = self.column.saturating_add($len);
                chars.next();
            }};
        }

        let is_valid_char = |ch: char, dots: bool| {
            if ch == '.' && !dots {
                return false;
            }

            ch.is_alphanumeric() || matches!(ch, '_' | '.' | '=' | '!' | '<' | '>' | '&' | '|')
        };

        while let Some(&ch) = chars.peek() {
            match ch {
                // Whitespace
                ' ' | '\t' => {
                    chars.next();
                    self.column = self.column.saturating_add(1);
                }
                '\n' => {
                    chars.next();
                    self.line = self.line.saturating_add(1);
                    self.column = 0;
                }
                // Comments / Slash operator
                '/' => {
                    // Look ahead to distinguish between comment vs path
                    if let Some(next_ch) = chars.clone().nth(1) {
                        if next_ch == '/' {
                            // Comment
                            chars.next();
                            chars.next();
                            self.column = self.column.saturating_add(2);
                            while let Some(&ch) = chars.peek() {
                                if ch == '\n' {
                                    break;
                                }
                                chars.next();
                                self.column = self.column.saturating_add(1);
                            }
                            continue;
                        }

                        let line_start = self.line;
                        let column_start = self.column;

                        // Path start
                        let path_token = {
                            let chars: &mut Peekable<Chars<'_>> = &mut chars;
                            let mut path_buf = String::new();
                            let mut interpolated_tokens = Vec::new();
                            let mut start_interpolation = false;

                            while let Some(&ch) = chars.peek() {
                                match ch {
                                    '"' | ' ' | '\n' | '\t' | ',' | ')' | '}' | ']' => break,
                                    '$' if chars.clone().nth(1) == Some('{') => {
                                        // Flush current path segment if any
                                        if !path_buf.is_empty() {
                                            interpolated_tokens.push(Token::new(
                                                TokenType::String(path_buf.clone()),
                                                Section::new(
                                                    self.line..=self.line,
                                                    self.column
                                                        ..=self
                                                            .column
                                                            .saturating_add(path_buf.len()),
                                                ),
                                            ));
                                            path_buf.clear();
                                        }

                                        // Consume `${`
                                        chars.next();
                                        chars.next();
                                        self.column = self.column.saturating_add(2);
                                        start_interpolation = true;

                                        let mut nested = String::new();
                                        let mut depth: i32 = 1;
                                        for nch in chars.by_ref() {
                                            self.column = self.column.saturating_add(1);
                                            match nch {
                                                '{' => depth = depth.saturating_add(1),
                                                '}' => {
                                                    depth = depth.saturating_sub(1);
                                                    if depth == 0 {
                                                        break;
                                                    }
                                                }
                                                _ => {}
                                            }
                                            nested.push(nch);
                                        }

                                        let mut nested_tokenizer = Self {
                                            source: nested.clone().into(),
                                            line: self.line,
                                            column: self.column,
                                        };
                                        let nested = nested_tokenizer.tokenize()?;
                                        interpolated_tokens.extend(nested);
                                    }
                                    _ => {
                                        path_buf.push(ch);
                                        chars.next();
                                        self.column = self.column.saturating_add(1);
                                    }
                                }
                            }

                            if start_interpolation {
                                if !path_buf.is_empty() {
                                    interpolated_tokens.push(Token::new(
                                        TokenType::String(path_buf),
                                        Section::new(
                                            line_start..=self.line,
                                            column_start..=self.column,
                                        ),
                                    ));
                                }
                                Ok::<TokenType, types::Error>(TokenType::InterpolatedPath(
                                    interpolated_tokens,
                                ))
                            } else {
                                Ok(TokenType::Path(PathBuf::from(path_buf)))
                            }
                        }?;

                        tokens.push(Token::new(
                            path_token,
                            Section::new(line_start..=self.line, column_start..=self.column),
                        ));
                        continue;
                    }

                    // Standalone slash
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
                '*' => push_token!(Multiply, 1),
                '%' => push_token!(Modulo, 1),

                // Misc
                ',' => push_token!(Comma, 1),
                ':' => push_token!(Colon, 1),
                '.' => {
                    if let Some(next_ch) = chars.clone().nth(1)
                        && matches!(next_ch, '/' | '.')
                    {
                        let line_start = self.line;
                        let column_start = self.column;

                        let path_token = {
                            let mut path_buf = String::new();
                            let mut interpolated_tokens = Vec::new();
                            let mut start_interpolation = false;

                            while let Some(&ch) = chars.peek() {
                                match ch {
                                    '"' | ' ' | '\n' | '\t' | ',' | ')' | '}' | ']' => break,
                                    '$' if chars.clone().nth(1) == Some('{') => {
                                        // Flush current path segment if any
                                        if !path_buf.is_empty() {
                                            interpolated_tokens.push(Token::new(
                                                TokenType::String(path_buf.clone()),
                                                Section::new(
                                                    self.line..=self.line,
                                                    self.column
                                                        ..=self
                                                            .column
                                                            .saturating_add(path_buf.len()),
                                                ),
                                            ));
                                            path_buf.clear();
                                        }

                                        // Consume `${`
                                        chars.next();
                                        chars.next();
                                        self.column = self.column.saturating_add(2);
                                        start_interpolation = true;

                                        let mut nested = String::new();
                                        let mut depth: i32 = 1;
                                        for nch in chars.by_ref() {
                                            self.column = self.column.saturating_add(1);
                                            match nch {
                                                '{' => depth = depth.saturating_add(1),
                                                '}' => {
                                                    depth = depth.saturating_sub(1);
                                                    if depth == 0 {
                                                        break;
                                                    }
                                                }
                                                _ => {}
                                            }
                                            nested.push(nch);
                                        }

                                        let mut nested_tokenizer = Self {
                                            source: nested.clone().into(),
                                            line: self.line,
                                            column: self.column,
                                        };
                                        let nested = nested_tokenizer.tokenize()?;
                                        interpolated_tokens.extend(nested);
                                    }
                                    _ => {
                                        path_buf.push(ch);
                                        chars.next();
                                        self.column = self.column.saturating_add(1);
                                    }
                                }
                            }

                            if start_interpolation {
                                if !path_buf.is_empty() {
                                    interpolated_tokens.push(Token::new(
                                        TokenType::String(path_buf),
                                        Section::new(
                                            line_start..=self.line,
                                            column_start..=self.column,
                                        ),
                                    ));
                                }
                                Ok::<TokenType, types::Error>(TokenType::InterpolatedPath(
                                    interpolated_tokens,
                                ))
                            } else {
                                Ok(TokenType::Path(PathBuf::from(path_buf)))
                            }
                        }?;

                        tokens.push(Token::new(
                            path_token,
                            Section::new(line_start..=self.line, column_start..=self.column),
                        ));
                        continue;
                    }

                    push_token!(Dot, 1);
                }

                // Strings
                #[allow(clippy::range_minus_one, reason = "Exclusive ranges can not be used")]
                '"' => {
                    let original_start = (self.line, self.column);
                    let mut start = (self.line, self.column);
                    let mut closed = false;
                    let mut values = Vec::new();
                    let mut buffer = String::new();

                    // Consume the opening quote
                    chars.next();
                    self.column = self.column.saturating_add(1);

                    while let Some(&ch) = chars.peek() {
                        match ch {
                            '"' => {
                                // Closing quote
                                chars.next();
                                self.column = self.column.saturating_add(1);
                                closed = true;
                                break;
                            }

                            '\\' => {
                                // Escape sequences
                                chars.next();
                                self.column = self.column.saturating_add(1);
                                if let Some(&escaped_char) = chars.peek() {
                                    buffer.push(escape(escaped_char));
                                    chars.next();
                                    self.column = self.column.saturating_add(1);
                                }
                            }

                            '$' => {
                                if chars.clone().nth(1) == Some('{') {
                                    // Flush current buffer to tokens
                                    if !buffer.is_empty() {
                                        values.push(Token::new(
                                            TokenType::String(buffer.clone()),
                                            Section::new(
                                                start.0..=self.line,
                                                start.1.saturating_add(1)..=self.column,
                                            ),
                                        ));
                                        buffer.clear();
                                    }

                                    // Consume `${`
                                    chars.next();
                                    chars.next();
                                    self.column = self.column.saturating_add(2);

                                    // Find the range of the interpolation
                                    let nested_start = (self.line, self.column);
                                    let mut nested_depth: i32 = 1;
                                    let mut nested_content = String::new();

                                    for nested_char in &mut chars {
                                        self.column = self.column.saturating_add(1);

                                        match nested_char {
                                            '{' => nested_depth = nested_depth.saturating_add(1),
                                            '}' => {
                                                nested_depth = nested_depth.saturating_sub(1);
                                                if nested_depth == 0 {
                                                    start.1 = self.column;
                                                    break;
                                                }
                                            }
                                            _ => {}
                                        }

                                        nested_content.push(nested_char);
                                    }

                                    // Ensure the interpolation is closed
                                    if nested_depth != 0 {
                                        return Err(Error::new(
                                            ErrorType::UnclosedInterpolation,
                                            Some(Location {
                                                path: self.source.path.clone(),
                                                text: self.source.text.clone(),
                                                section: Some(Section::new(
                                                    nested_start.0..=self.line,
                                                    nested_start.1..=self.column,
                                                )),
                                            }),
                                        ));
                                    }

                                    let mut nested_tokenizer = Self {
                                        source: nested_content.into(),
                                        line: self.line,
                                        column: nested_start.1,
                                    };

                                    let nested_tokens = nested_tokenizer.tokenize()?;

                                    if nested_tokens.len() == 1 {
                                        values.extend(nested_tokens);
                                    } else {
                                        values.push(Token::new(
                                            TokenType::InterpolatedString(nested_tokens),
                                            Section::new(
                                                nested_start.0..=self.line,
                                                nested_start.1..=self.column,
                                            ),
                                        ));
                                    }
                                } else {
                                    buffer.push('$');
                                    chars.next();
                                    self.column = self.column.saturating_add(1);
                                }
                            }

                            _ => {
                                // Regular characters
                                buffer.push(ch);
                                chars.next();
                                self.column = self.column.saturating_add(1);
                            }
                        }
                    }

                    // Flush remaining buffer to tokens
                    if !buffer.is_empty() {
                        values.push(Token::new(
                            TokenType::String(buffer.clone()),
                            Section::new(
                                start.0..=self.line,
                                start.1..=self.column.saturating_sub(1),
                            ),
                        ));
                    }

                    if !closed {
                        return Err(Error::new(
                            ErrorType::UnclosedString,
                            Some(Location {
                                path: self.source.path.clone(),
                                text: self.source.text.clone(),
                                section: Some(Section::new(
                                    original_start.0..=self.line,
                                    original_start.1..=self.column,
                                )),
                            }),
                        ));
                    }

                    if values.len() <= 1 {
                        tokens.push(Token::new(
                            TokenType::String(buffer),
                            Section::new(start.0..=self.line, start.1..=self.column),
                        ));
                    } else {
                        tokens.push(Token::new(
                            TokenType::InterpolatedString(values),
                            Section::new(
                                original_start.0..=self.line,
                                original_start.1..=self.column,
                            ),
                        ));
                    }
                }

                // Parse numbers and floats
                _ if ch.is_ascii_digit() || ch == '.' || ch == '-' => {
                    let start_line = self.line;
                    let mut value = String::new();

                    while let Some(&ch) = chars.peek()
                        && (ch.is_ascii_digit() || ch == '.' || ch == '-')
                    {
                        value.push(ch);
                        chars.next();

                        // If character is a bang break and the next character is not a equals.
                        // This is to properly handle the `Not` token.
                        if ch == '!' && chars.peek() != Some(&'=') {
                            break;
                        }
                    }

                    self.column = self.column.saturating_add(value.len());
                    match value.as_str() {
                        "-" => push_token!(Minus, 1),
                        _ if value.parse::<i64>().is_ok() => {
                            tokens.push(Token::new(
                                TokenType::Int(value.parse::<isize>()?),
                                Section::new(
                                    start_line..=self.line,
                                    self.column.saturating_sub(value.len())..=self.column,
                                ),
                            ));
                        }
                        _ if value.parse::<f64>().is_ok() => {
                            tokens.push(Token::new(
                                TokenType::Float(value.parse::<f64>()?),
                                Section::new(
                                    start_line..=self.line,
                                    self.column.saturating_sub(value.len())..=self.column,
                                ),
                            ));
                        }
                        _ => (),
                    }
                }

                // Multi-character tokens (literals, keywords, identifiers, operators)
                _ if is_valid_char(ch, false) => {
                    let start_line = self.line;
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
                                Section::new(
                                    start_line..=self.line,
                                    self.column.saturating_sub(value.len())..=self.column,
                                ),
                            ));
                        }};
                        ($token:ident($value:expr)) => {{
                            tokens.push(Token::new(
                                TokenType::$token($value),
                                Section::new(
                                    start_line..=self.line,
                                    self.column.saturating_sub(value.len())..=self.column,
                                ),
                            ));
                        }};
                    }

                    self.column = self.column.saturating_add(value.len());
                    match value.as_str() {
                        // Null
                        "null" => push_long_token!(Null),

                        // Boolean
                        "true" => push_long_token!(Bool(true)),
                        "false" => push_long_token!(Bool(false)),

                        // Keywords
                        "let" => push_long_token!(Let),

                        // Logic operators
                        "==" => push_long_token!(Eq),
                        "=" => push_long_token!(Equals),
                        "!=" => push_long_token!(NotEq),
                        "!" => push_long_token!(Not),
                        ">=" => push_long_token!(GtEq),
                        ">" => push_long_token!(Gt),
                        "<=" => push_long_token!(LtEq),
                        "<" => push_long_token!(Lt),
                        "&&" => push_long_token!(And),
                        "||" => push_long_token!(Or),

                        // Identifier
                        _ => push_long_token!(Identifier(value.to_string())),
                    }
                }

                _ => {
                    if let Some(path) = &self.source.path {
                        let location = Location::from_path(path.clone())?
                            .section(Section::new(self.line..=self.line, 0..=usize::MAX));
                        return Err(Error::new(ErrorType::UnexpectedToken(ch), Some(location)));
                    }

                    let location = Location {
                        path: None,
                        text: self.source.text.clone(),
                        section: Some(Section::new(self.line..=self.line, 0..=usize::MAX)),
                    };

                    return Err(Error::new(ErrorType::UnexpectedToken(ch), Some(location)));
                }
            }
        }

        Ok(tokens)
    }
}

fn escape(ch: char) -> char {
    match ch {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        '0' => '\0',
        _ => ch,
    }
}
