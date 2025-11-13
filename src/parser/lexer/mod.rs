use miette::{NamedSource, SourceSpan};
use std::{iter::Peekable, path::PathBuf, str::Chars};
use types::{Error, Token, TokenKind};

use crate::parser::lexer::types::ErrorKind;

pub mod types;

pub struct Lexer {
    pub(crate) source: NamedSource<String>,
    pub(crate) pos: usize,
}

impl Lexer {
    pub fn new(source: NamedSource<String>) -> Self {
        Self { source, pos: 0 }
    }

    /// Tokenizes the source code inside the [`Parser`] struct.
    /// # Errors
    /// This function will return an error if a tokenization error occurs.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();
        let mut chars = self.source.inner().chars().peekable();

        macro_rules! push_token {
            ($token:ident, $len:expr) => {{
                tokens.push(Token::new(TokenKind::$token, (self.pos, $len).into()));
                self.pos = self.pos.saturating_add($len);
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
                ' ' | '\t' | '\n' | '\r' => {
                    chars.next();
                    self.pos = self.pos.saturating_add(1);
                }
                // Comments / Slash operator
                '/' => {
                    // Look ahead to distinguish between comment vs path
                    if let Some(next_ch) = chars.clone().nth(1) {
                        if next_ch == '/' {
                            chars.next();
                            chars.next();
                            self.pos = self.pos.saturating_add(2);
                            while let Some(&ch) = chars.peek() {
                                if ch == '\n' {
                                    break;
                                }
                                chars.next();
                                self.pos = self.pos.saturating_add(1);
                            }
                            continue;
                        }

                        if next_ch == ' ' {
                            push_token!(Slash, 1);
                            continue;
                        }

                        let pos_start = self.pos;

                        let path_token = {
                            let chars: &mut Peekable<Chars<'_>> = &mut chars;
                            let mut path_buf = String::new();
                            let mut interpolated_tokens = Vec::new();
                            let mut start_interpolation = false;

                            while let Some(&ch) = chars.peek() {
                                match ch {
                                    '"' | ' ' | '\n' | '\t' | '\r' | ',' | ')' | '}' | ']' => break,
                                    '$' if chars.clone().nth(1) == Some('{') => {
                                        if !path_buf.is_empty() {
                                            interpolated_tokens.push(Token::new(
                                                TokenKind::String(path_buf.clone()),
                                                (self.pos, path_buf.len()).into(),
                                            ));
                                            path_buf.clear();
                                        }

                                        // Consume `${`
                                        chars.next();
                                        chars.next();
                                        self.pos = self.pos.saturating_add(2);
                                        start_interpolation = true;

                                        let mut nested = String::new();
                                        let mut depth: i32 = 1;
                                        for nch in chars.by_ref() {
                                            self.pos = self.pos.saturating_add(1);
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

                                        let mut nested_lexer = Self {
                                            source: NamedSource::new(self.source.name(), nested),
                                            pos: self.pos,
                                        };
                                        let nested = nested_lexer.tokenize()?;
                                        interpolated_tokens.extend(nested);
                                    }
                                    _ => {
                                        path_buf.push(ch);
                                        chars.next();
                                        self.pos = self.pos.saturating_add(1);
                                    }
                                }
                            }

                            if start_interpolation {
                                if !path_buf.is_empty() {
                                    let len = path_buf.len();
                                    interpolated_tokens.push(Token::new(
                                        TokenKind::String(path_buf),
                                        (pos_start, len).into(),
                                    ));
                                }
                                Ok(TokenKind::InterpolatedPath(interpolated_tokens))
                            } else {
                                Ok(TokenKind::Path(PathBuf::from(path_buf)))
                            }
                        }?;

                        tokens.push(Token::new(
                            path_token,
                            (pos_start, self.pos.saturating_sub(pos_start)).into(),
                        ));
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
                '*' => push_token!(Multiply, 1),
                '%' => push_token!(Modulo, 1),

                // Misc
                ',' => push_token!(Comma, 1),
                ':' => push_token!(Colon, 1),
                '.' => {
                    if let Some(next_ch) = chars.clone().nth(1)
                        && matches!(next_ch, '/' | '.')
                    {
                        let pos_start = self.pos;

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
                                                TokenKind::String(path_buf.clone()),
                                                (self.pos, path_buf.len()).into(),
                                            ));
                                            path_buf.clear();
                                        }

                                        // Consume `${`
                                        chars.next();
                                        chars.next();
                                        self.pos = self.pos.saturating_add(2);
                                        start_interpolation = true;

                                        let mut nested = String::new();
                                        let mut depth: i32 = 1;
                                        for nch in chars.by_ref() {
                                            self.pos = self.pos.saturating_add(1);
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

                                        let mut nested_lexer = Self {
                                            source: NamedSource::new(self.source.name(), nested),
                                            pos: self.pos,
                                        };
                                        let nested = nested_lexer.tokenize()?;
                                        interpolated_tokens.extend(nested);
                                    }
                                    _ => {
                                        path_buf.push(ch);
                                        chars.next();
                                        self.pos = self.pos.saturating_add(1);
                                    }
                                }
                            }

                            if start_interpolation {
                                if !path_buf.is_empty() {
                                    let len = path_buf.len();
                                    interpolated_tokens.push(Token::new(
                                        TokenKind::String(path_buf),
                                        (pos_start, len).into(),
                                    ));
                                }
                                Ok(TokenKind::InterpolatedPath(interpolated_tokens))
                            } else {
                                Ok(TokenKind::Path(PathBuf::from(path_buf)))
                            }
                        }?;

                        tokens.push(Token::new(
                            path_token,
                            (pos_start, self.pos.saturating_sub(pos_start)).into(),
                        ));
                        continue;
                    }

                    push_token!(Dot, 1);
                }

                // Strings
                #[allow(clippy::range_minus_one, reason = "Exclusive ranges can not be used")]
                '"' => {
                    let original_pos = self.pos;
                    let mut start = self.pos;
                    let mut closed = false;
                    let mut values = Vec::new();
                    let mut buffer = String::new();

                    chars.next();
                    self.pos = self.pos.saturating_add(1);

                    while let Some(&ch) = chars.peek() {
                        match ch {
                            '"' => {
                                chars.next();
                                self.pos = self.pos.saturating_add(1);
                                closed = true;
                                break;
                            }

                            '\\' => {
                                chars.next();
                                self.pos = self.pos.saturating_add(1);
                                if let Some(&escaped_char) = chars.peek() {
                                    buffer.push(escape(escaped_char));
                                    chars.next();
                                    self.pos = self.pos.saturating_add(1);
                                }
                            }

                            '$' => {
                                if chars.clone().nth(1) == Some('{') {
                                    if !buffer.is_empty() {
                                        values.push(Token::new(
                                            TokenKind::String(buffer.clone()),
                                            (start.saturating_add(1), buffer.len()).into(),
                                        ));
                                        buffer.clear();
                                    }

                                    // Consume `${`
                                    chars.next();
                                    chars.next();
                                    self.pos = self.pos.saturating_add(2);

                                    let nested_start = self.pos;
                                    let mut nested_depth: i32 = 1;
                                    let mut nested_content = String::new();

                                    for nested_char in &mut chars {
                                        self.pos = self.pos.saturating_add(1);

                                        match nested_char {
                                            '{' => nested_depth = nested_depth.saturating_add(1),
                                            '}' => {
                                                nested_depth = nested_depth.saturating_sub(1);
                                                if nested_depth == 0 {
                                                    start = self.pos;
                                                    break;
                                                }
                                            }
                                            _ => {}
                                        }

                                        nested_content.push(nested_char);
                                    }

                                    if nested_depth != 0 {
                                        return Err(Error::new(
                                            ErrorKind::UnclosedInterpolation,
                                            self.source.clone(),
                                            (nested_start, self.pos.saturating_sub(nested_start))
                                                .into(),
                                        ));
                                    }

                                    let mut nested_lexer = Self {
                                        source: NamedSource::new(
                                            self.source.name(),
                                            nested_content,
                                        ),
                                        pos: nested_start,
                                    };

                                    let nested_tokens = nested_lexer.tokenize()?;

                                    if nested_tokens.len() == 1 {
                                        values.extend(nested_tokens);
                                    } else {
                                        values.push(Token::new(
                                            TokenKind::InterpolatedString(nested_tokens),
                                            (nested_start, self.pos.saturating_sub(nested_start))
                                                .into(),
                                        ));
                                    }
                                } else {
                                    buffer.push('$');
                                    chars.next();
                                    self.pos = self.pos.saturating_add(1);
                                }
                            }

                            _ => {
                                buffer.push(ch);
                                chars.next();
                                self.pos = self.pos.saturating_add(1);
                            }
                        }
                    }

                    if !buffer.is_empty() {
                        values.push(Token::new(
                            TokenKind::String(buffer.clone()),
                            (start, self.pos.saturating_sub(start.saturating_add(1))).into(),
                        ));
                    }

                    if !closed {
                        return Err(Error::new(
                            ErrorKind::UnclosedString,
                            self.source.clone(),
                            (original_pos, self.pos.saturating_sub(original_pos)).into(),
                        ));
                    }

                    if values.len() <= 1 {
                        tokens.push(Token::new(
                            TokenKind::String(buffer),
                            (start, self.pos.saturating_sub(start)).into(),
                        ));
                    } else {
                        tokens.push(Token::new(
                            TokenKind::InterpolatedString(values),
                            (original_pos, self.pos.saturating_sub(original_pos)).into(),
                        ));
                    }
                }

                // Parse numbers and floats
                _ if ch.is_ascii_digit() || ch == '.' || ch == '-' => {
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

                    self.pos = self.pos.saturating_add(value.len());
                    match value.as_str() {
                        "-" => push_token!(Minus, 1),
                        _ if value.parse::<i64>().is_ok() => {
                            tokens.push(Token::new(
                                TokenKind::Int(value.parse::<isize>().map_err(|error| {
                                    Error::new(
                                        ErrorKind::ParseIntError(error),
                                        self.source.clone(),
                                        (self.pos.saturating_sub(value.len()), value.len()).into(),
                                    )
                                })?),
                                (self.pos.saturating_sub(value.len()), value.len()).into(),
                            ));
                        }
                        _ if value.parse::<f64>().is_ok() => {
                            tokens.push(Token::new(
                                TokenKind::Float(value.parse::<f64>().map_err(|error| {
                                    Error::new(
                                        ErrorKind::ParseFloatError(error),
                                        self.source.clone(),
                                        (self.pos.saturating_sub(value.len()), value.len()).into(),
                                    )
                                })?),
                                (self.pos.saturating_sub(value.len()), value.len()).into(),
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
                                TokenKind::$token,
                                (self.pos.saturating_sub(value.len()), value.len()).into(),
                            ));
                        }};
                        ($token:ident($value:expr)) => {{
                            tokens.push(Token::new(
                                TokenKind::$token($value),
                                (self.pos.saturating_sub(value.len()), value.len()).into(),
                            ));
                        }};
                    }

                    self.pos = self.pos.saturating_add(value.len());
                    match value.as_str() {
                        // Null
                        "null" => push_long_token!(Null),

                        // Booleans
                        "true" => push_long_token!(Bool(true)),
                        "false" => push_long_token!(Bool(false)),

                        // Keywords
                        "let" => push_long_token!(Let),
                        "in" => push_long_token!(In),

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
                        _ => push_long_token!(Identifier(value.clone())),
                    }
                }

                _ => {
                    return Err(Error::new(
                        ErrorKind::UnexpectedToken,
                        self.source.clone(),
                        SourceSpan::new(self.pos.saturating_sub(1).into(), 1),
                    ));
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
