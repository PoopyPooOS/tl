#![allow(clippy::unused_self)]

use super::types::{BinaryOperator, Expr, Literal, Statement};
use crate::{
    parser::tokenizer::types::{Token, TokenType},
    source::Source,
};
use logger::{make_error, Location, Log};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    source: Source,
    position: usize,
}

pub type StatementResult = Result<Vec<Statement>, Box<Log>>;
pub type ExprResult = Result<Expr, Box<Log>>;

impl Parser {
    pub fn new(tokens: Vec<Token>, source: impl Into<Source>) -> Self {
        Self {
            tokens,
            source: source.into(),
            position: 0,
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
                TokenType::Identifier(_) => self.parse_ident()?,
                _ => vec![Statement::Expr(self.parse_expr()?)],
            };

            statements.extend(parsed);
        }
        Ok(statements)
    }

    /// Parses a let statement
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

    /// Parse function calls and simple identifiers
    fn parse_ident(&mut self) -> StatementResult {
        let mut is_call = false;

        if self
            .tokens
            .get(self.position + 1)
            .is_some_and(|token| token.token_type == TokenType::LParen)
        {
            is_call = true;
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
                    return Err(Box::new(
                        make_error!("Expected binary operator", location: Location::new(self.source.path.clone(), next_token.line..=next_token.line)),
                    ));
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

    fn parse_literal(&mut self) -> ExprResult {
        let token = &self.tokens[self.position];

        let expr = match &token.token_type {
            TokenType::String(v) => Expr::Literal(Literal::String(v.clone())),
            TokenType::Number(v) => Expr::Literal(Literal::Number(*v)),
            TokenType::Float(v) => Expr::Literal(Literal::Float(*v)),
            TokenType::Bool(v) => Expr::Literal(Literal::Bool(*v)),
            TokenType::Identifier(v) => Expr::Identifier(v.clone()),
            other => {
                return Err(Box::new(
                    make_error!(format!("Unexpected token found in expression: {other}"), location: Location::new_with_section(self.source.path.clone(), token.line..=token.line, token.column..=token.column + token.len)),
                ));
            }
        };

        self.position += 1;

        Ok(expr)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn consume(&mut self, expected: TokenType) -> Result<(), Box<Log>> {
        match self.advance() {
            Some(token) if token.token_type == expected => Ok(()),
            _ => Err(Box::new(make_error!(format!("Expected token: {expected:#?}")))),
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

// #[allow(clippy::unnecessary_wraps)]
// impl Parser {
//     fn parse_expr(&self, starting_position: usize) -> Result<ParseResult, Box<Log>> {
//         let mut result = ParseResult {
//             advance_by: 1, // Expr
//             ..Default::default()
//         };

//         let token = &self.tokens[starting_position];

//         let next_token = self.tokens.get(starting_position + result.advance_by);

//         if let Some(next_token) = next_token {
//             let binary_op = match next_token.token_type {
//                 TokenType::Plus | TokenType::Minus | TokenType::Multiply | TokenType::Slash => {
//                     Some(self.parse_binary_op(starting_position)?)
//                 }
//                 _ => None,
//             };

//             if let Some(binary_op) = binary_op {
//                 result.advance_by += binary_op.advance_by;

//                 if let Some(Statement::Expr(expr)) = binary_op.statements.first() {
//                     result.statements.push(Statement::Expr(expr.clone()));
//                 } else {
//                     return Err(Box::new(
//                         make_error!("Failed to parse binary operation", location: Location::new(&self.path, token.line..=token.line)),
//                     ));
//                 }

//                 return Ok(result);
//             }
//         }

//         let parsed = self.parse_literal(starting_position)?;
//         let expr = parsed.statements.first();

//         if let Some(Statement::Expr(expr)) = expr {
//             result.statements.push(Statement::Expr(expr.clone()));
//         } else {
//             return Err(Box::new(
//                 make_error!("Failed to parse expression", location: Location::new(&self.path, token.line..=token.line)),
//             ));
//         }

//         Ok(result)
//     }

//     fn parse_let(&self, starting_position: usize) -> Result<ParseResult, Box<Log>> {
//         let mut result = ParseResult {
//             advance_by: 1, // Let token
//             ..Default::default()
//         };

//         let token = &self.tokens[starting_position];
//         let mut name = String::new();

//         {
//             let next_token = self.tokens.get(starting_position + result.advance_by);

//             if next_token.is_none() {
//                 return Err(Box::new(
//                     make_error!("Expected identifier after 'let'", location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
//                 ));
//             }

//             match &next_token.unwrap().token_type {
//                 TokenType::Identifier(ident) => {
//                     name.clone_from(ident);
//                     result.advance_by += 1;
//                 }
//                 _ => {
//                     return Err(Box::new(
//                         make_error!("Expected identifier after 'let'", location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
//                     ));
//                 }
//             }
//         }

//         {
//             let previous_token = &self.tokens[(starting_position + result.advance_by) - 1];
//             let next_token = self.tokens.get(starting_position + result.advance_by);

//             if next_token.is_none() {
//                 return Err(Box::new(
//                     make_error!("Expected '=' after identifier in variable declaration", location: Location::new_with_section(&self.path, previous_token.line..=previous_token.line, previous_token.column..=previous_token.column + previous_token.len)),
//                 ));
//             }

//             match &next_token.unwrap().token_type {
//                 TokenType::Equals => result.advance_by += 1,
//                 _ => {
//                     return Err(Box::new(
//                         make_error!("Expected '=' after identifier in variable declaration", location: Location::new_with_section(&self.path, previous_token.line..=previous_token.line, previous_token.column..=previous_token.column + previous_token.len)),
//                     ));
//                 }
//             }
//         }

//         let value = self.parse_expr(starting_position + result.advance_by)?;
//         result.advance_by += value.advance_by;

//         if value.statements.len() != 1 {
//             let token = &self.tokens[starting_position + result.advance_by - 1];
//             return Err(Box::new(
//                 make_error!("Expected single expression in variable declaration", location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
//             ));
//         }

//         if let Statement::Expr(expr) = &value.statements[0] {
//             result.statements.push(Statement::Let { name, value: expr.clone() });
//         }

//         Ok(result)
//     }

//     fn parse_binary_op(&self, starting_position: usize) -> Result<ParseResult, Box<Log>> {
//         let mut result = ParseResult {
//             advance_by: 3, // Binary OP
//             ..Default::default()
//         };

//         let left = &self.tokens[starting_position];
//         let operator = &self.tokens.get(starting_position + 1);
//         let right = &self.tokens.get(starting_position + 2);

//         if operator.is_none() {
//             return Err(Box::new(
//                 make_error!("Expected binary operator", location: Location::new_with_section(&self.path, left.line..=left.line, left.column..=left.column + left.len)),
//             ));
//         }

//         let operator = operator.unwrap();

//         if right.is_none() {
//             return Err(Box::new(
//                 make_error!("Expected right side of binary operation", location: Location::new_with_section(&self.path, operator.line..=operator.line, operator.column..=operator.column + operator.len)),
//             ));
//         }

//         let operator = match &operator.token_type {
//             TokenType::Plus => BinaryOperator::Plus,
//             TokenType::Minus => BinaryOperator::Minus,
//             TokenType::Multiply => BinaryOperator::Multiply,
//             TokenType::Slash => BinaryOperator::Divide,
//             other => {
//                 return Err(Box::new(make_error!(format!("'{other}' is not a valid binary operator"))));
//             }
//         };

//         let left = self.parse_literal(starting_position)?;
//         result.advance_by += left.advance_by;
//         let right = self.parse_literal(starting_position + 2)?;
//         result.advance_by += right.advance_by;

//         if left.statements.len() != 1 {
//             let token = &self.tokens[starting_position];
//             return Err(Box::new(
//                 make_error!("Expected single expression in binary operation", location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
//             ));
//         }

//         if right.statements.len() != 1 {
//             let token = &self.tokens[starting_position + 2];
//             return Err(Box::new(
//                 make_error!("Expected single expression in binary operation", location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
//             ));
//         }

//         if let Statement::Expr(left) = &left.statements[0] {
//             if let Statement::Expr(right) = &right.statements[0] {
//                 result.statements.push(Statement::Expr(Expr::BinaryOp(BinaryOp {
//                     left: Box::new(left.clone()),
//                     operator,
//                     right: Box::new(right.clone()),
//                 })));
//             }
//         }

//         Ok(result)
//     }

//     fn parse_literal(&self, starting_position: usize) -> Result<ParseResult, Box<Log>> {
//         let mut result = ParseResult {
//             advance_by: 1, // Expr
//             ..Default::default()
//         };

//         let token = &self.tokens[starting_position];

//         let expr = match &token.token_type {
//             TokenType::String(v) => Expr::Literal(Literal::String(v.clone())),
//             TokenType::Number(v) => Expr::Literal(Literal::Number(*v)),
//             TokenType::Float(v) => Expr::Literal(Literal::Float(*v)),
//             TokenType::Bool(v) => Expr::Literal(Literal::Bool(*v)),
//             TokenType::Identifier(v) => Expr::Identifier(v.clone()),
//             other => {
//                 return Err(Box::new(
//                     make_error!(format!("Unexpected token found in expression: {other}"), location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
//                 ));
//             }
//         };

//         result.statements.push(Statement::Expr(expr));

//         Ok(result)
//     }

//     // fn parse_binary(&mut self, min_precedence: u8) -> Result<Expr, Log> {
//     //     let mut left = self.parse_literal()?;

//     //     while let Some(op_token) = self.peek() {
//     //         let precedence = self.get_precedence(&op_token.token_type);
//     //         if precedence < min_precedence {
//     //             break;
//     //         }

//     //         let operator_token = {
//     //             let token = self.tokens.get(self.position);
//     //             if token.is_some() {
//     //                 self.position += 1;
//     //             }
//     //             token
//     //         }
//     //         .unwrap();
//     //         let operator = match &operator_token.token_type {
//     //             TokenType::Plus => BinaryOperator::Plus,
//     //             TokenType::Minus => BinaryOperator::Minus,
//     //             TokenType::Multiply => BinaryOperator::Multiply,
//     //             TokenType::Slash => BinaryOperator::Divide,
//     //             other => {
//     //                 error!(format!("'{other}' is not a valid binary operator"), location: Location::new_with_section(&self.path, operator_token.line..=operator_token.line, operator_token.column..=operator_token.column + operator_token.len));
//     //                 process::exit(0);
//     //             }
//     //         };

//     //         let right = self.parse_binary(precedence + 1)?;
//     //         left = Expr::BinaryOp {
//     //             left: Box::new(left),
//     //             operator,
//     //             right: Box::new(right),
//     //         };
//     //     }

//     //     Ok(left)
//     // }

//     fn get_precedence(&self, token: &TokenType) -> u8 {
//         match token {
//             TokenType::Plus | TokenType::Minus => 1,
//             TokenType::Multiply | TokenType::Slash => 2,
//             _ => 0,
//         }
//     }
// }
