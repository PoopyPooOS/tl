#![allow(
    unreachable_patterns,
    clippy::match_wildcard_for_single_variants,
    reason = "More expressions/statements may be added soon and im not rewriting the wildcard pattern again when they are added"
)]

use crate::parser::ast::types::{BinaryOperator, Expr, Literal, Statement};
use logger::{make_error, Log};
use std::{collections::HashMap, rc::Rc};
use types::Value;

mod stdlib;
pub mod types;

#[derive(Debug, Default)]
pub struct Scope {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Value>,
    scopes: HashMap<usize, Scope>,
    ast: Rc<Vec<Statement>>,
}

impl Scope {
    #[must_use]
    pub fn new(ast: Vec<Statement>) -> Self {
        Self {
            ast: Rc::new(ast),
            ..Default::default()
        }
    }

    /// Evaluates a list of statements (an AST).
    /// # Errors
    /// This function will return an error if an evaluation error occurs.
    pub fn eval(&mut self) -> Result<Option<Value>, Box<Log>> {
        let mut value: Option<Value> = None;

        self.init_stdlib();

        let ast_clone = Rc::clone(&self.ast);
        for node in ast_clone.iter() {
            value = match node {
                Statement::Expr(expr) => Some(self.eval_expr(expr)?),
                Statement::Let { name, value } => {
                    self.eval_let(name, value)?;
                    None
                }
                Statement::Fn { name, parameters, body } => {
                    self.eval_fn(name, parameters.clone(), body.clone());
                    None
                }
                _ => return Err(Box::new(make_error!(format!("Can not evaluate statement: {:#?}", node)))),
            };
        }
        Ok(value)
    }

    fn eval_fn(&mut self, name: impl Into<String>, parameters: Vec<String>, body: Vec<Statement>) {
        self.functions.insert(name.into(), Value::Function { parameters, body });
    }

    fn eval_call(&mut self, name: impl Into<String>, args: &[Expr]) -> Result<Option<Value>, Box<Log>> {
        let name: String = name.into();

        // Fetch the function from the scope
        let (parameters, body) = match self.functions.get(&name) {
            Some(Value::Function { parameters, body }) => (parameters.clone(), body.clone()),
            Some(Value::NativeFunction { body, .. }) => {
                return Ok(body(args.iter().map(|arg| self.eval_expr(arg)).collect::<Result<Vec<_>, _>>()?));
            }
            _ => return Err(Box::new(make_error!(format!("'{name}' does not exist or it is not a function")))),
        };

        // Check the number of arguments
        if parameters.len() != args.len() {
            return Err(Box::new(make_error!(format!(
                "Function '{name}' has {} parameters, but {} arguments were provided",
                parameters.len(),
                args.len()
            ))));
        }

        // Evaluate all arguments
        let evaluated_args: Vec<Value> = args.iter().map(|arg| self.eval_expr(arg)).collect::<Result<Vec<_>, _>>()?;

        // Create scope for function to be evaluated in
        let scope = self.create_scope(body);

        // Move arguments into scope
        for (param, value) in parameters.iter().zip(evaluated_args) {
            scope.variables.insert(param.clone(), value);
        }

        scope.eval()
    }

    fn eval_let(&mut self, name: impl Into<String>, value: &Expr) -> Result<(), Box<Log>> {
        let value = self.eval_expr(value)?;
        self.variables.insert(name.into(), value);
        Ok(())
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, Box<Log>> {
        match expr {
            Expr::Literal(literal) => Ok(self.eval_literal(literal)?),
            Expr::Identifier(ident) => self
                .variables
                .get(ident)
                .or_else(|| self.functions.get(ident))
                .cloned()
                .ok_or_else(|| Box::new(make_error!(format!("Variable '{ident}' does not exist")))),
            Expr::BinaryOp { left, operator, right } => {
                let left = self.eval_expr(left)?;
                let right = self.eval_expr(right)?;
                match operator {
                    BinaryOperator::Plus => Ok(left + right),
                    BinaryOperator::Minus => Ok(left - right),
                    BinaryOperator::Multiply => Ok(left * right),
                    BinaryOperator::Divide => Ok(left / right),
                }
            }
            Expr::Call { name, args } => match self.eval_call(name, args)? {
                Some(value) => Ok(value),
                None => Ok(Value::Null),
            },
        }
    }

    fn eval_literal(&mut self, literal: &Literal) -> Result<Value, Box<Log>> {
        match literal {
            Literal::Number(v) => Ok((*v).into()),
            Literal::Float(v) => Ok((*v).into()),
            Literal::Bool(v) => Ok((*v).into()),
            Literal::String(v) => Ok(v.to_string().into()),
            Literal::InterpolatedString(v) => {
                let mut string = String::new();

                for expr in v {
                    match expr {
                        Expr::Literal(v) => string.push_str(self.eval_literal(v)?.to_string().as_str()),
                        Expr::Identifier(ident) => {
                            let ident = ident.clone();
                            let evaluated = self.eval_expr(&Expr::Identifier(ident))?;
                            string.push_str(evaluated.to_string().as_str());
                        }
                        Expr::BinaryOp { left, operator, right } => {
                            let left = self.eval_expr(left)?;
                            let right = self.eval_expr(right)?;

                            match operator {
                                BinaryOperator::Plus => string.push_str((left + right).to_string().as_str()),
                                other => return Err(Box::new(make_error!(format!("Can not use operator '{other}' on a string")))),
                            }
                        }
                        _ => {
                            return Err(Box::new(make_error!(format!(
                                "Can not interpolate string with expression: {:#?}",
                                expr
                            ))))
                        }
                    }
                }

                Ok(string.into())
            }
            Literal::Array(v) => {
                let mut array = Vec::new();
                for item in v {
                    array.push(self.eval_expr(item)?);
                }
                Ok(array.into())
            }
            Literal::Object(v) => {
                let mut object = HashMap::new();
                for (key, value) in v {
                    object.insert(key.to_string(), self.eval_expr(value)?);
                }
                Ok(object.into())
            }
        }
    }

    fn create_scope(&mut self, ast: Vec<Statement>) -> &mut Scope {
        let scope_id = self.scopes.len();
        self.scopes.insert(scope_id, Scope::new(ast));
        self.scopes.get_mut(&scope_id).unwrap()
    }
}
