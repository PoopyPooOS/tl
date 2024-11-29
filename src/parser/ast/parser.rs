#![allow(clippy::unused_self)]

use std::collections::HashMap;

use super::types::{BinaryOperator, Expr, Literal, Statement};
use crate::{
    parser::tokenizer::{
        self,
        types::{Token, TokenType},
    },
    source::Source,
};
use logger::{make_error, Location, Log};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    source: Source,
    position: usize,

    context: Context,
}

#[derive(Debug, PartialEq, Clone)]
enum Context {
    TopLevel,
    Function,
    Object,
}

pub type StatementResult = Result<Vec<Statement>, Box<Log>>;
pub type ExprResult = Result<Expr, Box<Log>>;

impl Parser {
    pub fn new(tokens: Vec<Token>, source: impl Into<Source>) -> Self {
        Self {
            tokens,
            source: source.into(),
            position: 0,

            context: Context::TopLevel,
        }
    }

    /// Generates an AST based on the tokens of this [`Parser`].
    /// # Errors
    /// This function will return an error if a AST generation error occurs.
    pub fn parse(&mut self) -> StatementResult {
        let mut statements = Vec::new();
        while let Some(token) = self.tokens.get(self.position) {
            let parsed = match token.token_type {
                TokenType::Let => self.parse_let()?,
                TokenType::Fn => self.parse_fn()?,
                TokenType::Identifier(_) => self.parse_ident()?,
                TokenType::RBrace if self.context == Context::Function => break,
                _ => vec![Statement::Expr(self.parse_expr()?)],
            };

            statements.extend(parsed);
        }
        Ok(statements)
    }

    fn parse_fn(&mut self) -> StatementResult {
        self.consume(TokenType::Fn)?;
        let name = match self.advance() {
            Some(token) => {
                if let TokenType::Identifier(name) = &token.token_type {
                    name.clone()
                } else {
                    return Err(Box::new(make_error!("Expected identifier after 'fn'")));
                }
            }
            _ => return Err(Box::new(make_error!("Expected identifier after 'fn'"))),
        };

        // Parse arguments
        let mut parameters = Vec::new();
        self.consume(TokenType::LParen)?;

        // Handle empty parameters
        if let Some(token) = self.tokens.get(self.position)
            && token.token_type == TokenType::RParen
        {
            self.position += 1;
        } else {
            // Handle 1 argument that doesnt have commas
            let token = self.parse_ident()?;
            let token = match token.first().unwrap() {
                Statement::Expr(Expr::Identifier(name)) => name.clone(),
                _ => return Err(Box::new(make_error!("Expected identifier in 'fn' arguments"))),
            };

            parameters.push(token);

            // Handle multiple parameters
            while let Some(next_token) = self.tokens.get(self.position) {
                if next_token.token_type == TokenType::RParen {
                    self.consume(TokenType::RParen)?;
                    break;
                }

                self.consume(TokenType::Comma)?;

                let token = self.parse_ident()?;
                let token = match token.first().unwrap() {
                    Statement::Expr(Expr::Identifier(name)) => name.clone(),
                    _ => return Err(Box::new(make_error!("Expected identifier in 'fn' arguments"))),
                };

                parameters.push(token);
            }
        }

        // Parse function body
        self.consume(TokenType::LBrace)?;
        self.context = Context::Function;
        let body = self.parse()?;
        self.consume(TokenType::RBrace)
            .map_err(|_| Box::new(make_error!("Expected '}' after function body")))?;

        Ok(vec![Statement::Fn { name, parameters, body }])
    }

    fn parse_let(&mut self) -> StatementResult {
        self.consume(TokenType::Let)?;
        let name = match self.advance() {
            Some(token) => {
                if let TokenType::Identifier(name) = &token.token_type {
                    name.clone()
                } else {
                    return Err(Box::new(make_error!("Expected identifier after 'let'")));
                }
            }
            _ => return Err(Box::new(make_error!("Expected identifier after 'let'"))),
        };

        self.consume(TokenType::Equals)?;
        let value = self.parse_expr()?;

        Ok(vec![Statement::Let { name, value }])
    }

    /// Parse function calls and identifiers
    fn parse_ident(&mut self) -> StatementResult {
        let mut is_call = false;

        if let Some(next_token) = self.tokens.get(self.position + 1) {
            if next_token.token_type.is_binary_operator() {
                return self.parse_binary_op(0).map(|expr| vec![Statement::Expr(expr)]);
            }

            is_call = next_token.token_type == TokenType::LParen;
        }

        let name = match self.advance() {
            Some(token) => {
                if let TokenType::Identifier(name) = &token.token_type {
                    name.clone()
                } else {
                    return Err(Box::new(make_error!("Expected identifier after 'let'")));
                }
            }
            _ => return Err(Box::new(make_error!("Expected identifier after 'let'"))),
        };

        if !is_call {
            return Ok(vec![Statement::Expr(Expr::Identifier(name))]);
        }

        // Parse args
        let mut call_args = Vec::new();
        self.consume(TokenType::LParen)?;

        // Handle empty args
        if let Some(token) = self.tokens.get(self.position) {
            if token.token_type == TokenType::RParen {
                self.position += 1;
                return Ok(vec![Statement::Call { name, args: call_args }]);
            }
        }

        // Handle 1 argument that doesnt have commas
        let token = self.parse_expr()?;
        call_args.push(token);

        // Handle multiple args
        while let Some(next_token) = self.tokens.get(self.position) {
            if next_token.token_type == TokenType::RParen {
                break;
            }

            self.consume(TokenType::Comma)?;

            let token = self.parse_expr()?;
            call_args.push(token);
        }

        self.consume(TokenType::RParen)?;

        Ok(vec![Statement::Call { name, args: call_args }])
    }

    fn parse_expr(&mut self) -> ExprResult {
        let next_token = self.tokens.get(self.position + 1);

        if let Some(next_token) = next_token {
            let binary_op = match next_token.token_type {
                TokenType::Plus | TokenType::Minus | TokenType::Multiply | TokenType::Slash => Some(self.parse_binary_op(0)?),
                _ => None,
            };

            if let Some(binary_op) = binary_op {
                return Ok(binary_op);
            }
        }

        let expr = match &self.tokens[self.position].token_type {
            TokenType::LBrace => Some(self.parse_object()?),
            TokenType::LBracket => Some(self.parse_array()?),
            _ => None,
        };

        if let Some(expr) = expr {
            return Ok(expr);
        }

        self.parse_literal()
    }

    fn parse_binary_op(&mut self, min_precedence: u8) -> ExprResult {
        let mut left = self.parse_literal()?;

        while let Some(next_token) = self.tokens.get(self.position)
            && next_token.token_type.is_binary_operator()
        {
            let precedence = self.get_precedence(&next_token.token_type);
            if precedence < min_precedence {
                break;
            }

            let operator_token = {
                if let Some(operator_token) = self.tokens.get(self.position) {
                    self.position += 1;
                    operator_token
                } else {
                    let location = Location {
                        path: self.source.path.clone(),
                        text: if self.source.path.is_none() {
                            Some(self.source.text.clone())
                        } else {
                            None
                        },
                        lines: next_token.line..=next_token.line,
                        section: Some(next_token.column..=next_token.column + next_token.len),
                    };

                    return Err(Box::new(make_error!("Expected binary operator", location: location)));
                }
            };

            let operator = match &operator_token.token_type {
                TokenType::Plus => BinaryOperator::Plus,
                TokenType::Minus => BinaryOperator::Minus,
                TokenType::Multiply => BinaryOperator::Multiply,
                TokenType::Slash => BinaryOperator::Divide,
                other => {
                    return Err(Box::new(
                        make_error!(format!("'{other}' is not a valid binary operator"), location: Location::new_with_section(self.source.path.clone(), operator_token.line..=operator_token.line, operator_token.column..=operator_token.column + operator_token.len)),
                    ));
                }
            };

            let right = self.parse_binary_op(precedence + 1)?;

            left = Expr::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_object(&mut self) -> ExprResult {
        self.consume(TokenType::LBrace)?;

        let last_context = self.context.clone();
        self.context = Context::Object;
        let mut object: HashMap<String, Expr> = HashMap::new();

        while let Some(next_token) = self.tokens.get(self.position) {
            if next_token.token_type == TokenType::RBrace {
                self.consume(TokenType::RBrace)?;
                break;
            }

            let name = match &next_token.token_type {
                TokenType::Identifier(name) => name,
                _ => return Err(Box::new(make_error!("Expected key identifier in object"))),
            }
            .to_string();

            self.position += 1;

            // Support either "key: value" or "key = value"
            match self.advance() {
                Some(token) => if matches!(token.token_type, TokenType::Colon | TokenType::Equals) {},
                None => return Err(Box::new(make_error!("Expected ':' or '=' after object key"))),
            }

            let value = self.parse_expr()?;
            object.insert(name, value);
        }

        self.context = last_context;

        Ok(Expr::Literal(Literal::Object(object)))
    }

    fn parse_array(&mut self) -> ExprResult {
        self.consume(TokenType::LBracket)?;

        let mut array = Vec::new();
        while let Some(next_token) = self.tokens.get(self.position).cloned() {
            if next_token.token_type == TokenType::RBracket {
                self.consume(TokenType::RBracket)?;
                break;
            }

            let expr = self.parse_expr()?;
            array.push(expr);
        }

        Ok(Expr::Literal(Literal::Array(array)))
    }

    fn parse_literal(&mut self) -> ExprResult {
        let token = self.tokens[self.position].clone();

        let expr = match &token.token_type {
            TokenType::String(v) => self.parse_string(v)?,
            TokenType::Number(v) => Expr::Literal(Literal::Number(*v)),
            TokenType::Float(v) => Expr::Literal(Literal::Float(*v)),
            TokenType::Bool(v) => Expr::Literal(Literal::Bool(*v)),
            TokenType::Identifier(v) => Expr::Identifier(v.clone()),
            other => {
                let location = Location {
                    path: self.source.path.clone(),
                    text: if self.source.path.is_none() {
                        Some(self.source.text.clone())
                    } else {
                        None
                    },
                    lines: token.line..=token.line,
                    section: Some(token.column..=token.column + token.len),
                };

                return Err(Box::new(
                    make_error!(format!("Unexpected token found in expression: {other}"), location: location),
                ));
            }
        };

        self.position += 1;

        Ok(expr)
    }

    fn parse_string(&mut self, v: impl Into<String>) -> ExprResult {
        let v: String = v.into();
        let mut result = Vec::new();
        let mut start = 0;

        while let Some(open) = v[start..].find("${") {
            let open_pos = start + open;

            if open_pos > start {
                let literal_part = &v[start..open_pos];
                result.push(Expr::Literal(Literal::String(literal_part.to_string())));
            }

            if let Some(close) = v[open_pos..].find('}') {
                let close_pos = open_pos + close;
                let content = &v[open_pos + 2..close_pos];

                let source = Source::new(content);
                let tokens = tokenizer::Parser::new(&source).tokenize()?;
                let expr = Self::new(tokens, source).parse_expr()?;
                result.push(expr);

                start = close_pos + 1;
            } else {
                break;
            }
        }

        if start < v.len() {
            let remaining_part = &v[start..];
            result.push(Expr::Literal(Literal::String(remaining_part.to_string())));
        }

        if result.len() == 1 {
            if let Expr::Literal(Literal::String(s)) = &result[0] {
                return Ok(Expr::Literal(Literal::String(s.clone())));
            }
        }

        Ok(Expr::Literal(Literal::InterpolatedString(result)))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn consume(&mut self, expected: TokenType) -> Result<(), Box<Log>> {
        match self.advance() {
            Some(token) => {
                if token.token_type == expected {
                    Ok(())
                } else {
                    Err(Box::new(make_error!(format!(
                        "Expected token '{expected}' found '{}'",
                        token.token_type
                    ))))
                }
            }
            _ => Err(Box::new(make_error!(format!("Expected token: '{expected}'")))),
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    fn get_precedence(&self, token: &TokenType) -> u8 {
        match token {
            TokenType::Plus | TokenType::Minus => 1,
            TokenType::Multiply | TokenType::Slash => 2,
            _ => 0,
        }
    }
}
