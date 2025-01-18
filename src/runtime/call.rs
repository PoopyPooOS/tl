use super::{
    types::{Error, ErrorType, NativeFunction, Value},
    ValueResult,
};
use crate::parser::ast::types::Expr;

impl super::Scope {
    pub(super) fn eval_call(&mut self, name: &impl ToString, args: &[Expr]) -> ValueResult {
        let name = name.to_string();

        let mut evaluated_args = Vec::new();
        for expr in args {
            evaluated_args.push(self.eval_expr(expr)?);
        }
        let evaluated_args = evaluated_args;

        if let Some(native_fn) = self.native_functions.get(&name) {
            match native_fn {
                NativeFunction::Strict { params, func } => {
                    if args.len() != *params {
                        return Err(Box::new(Error::new(
                            ErrorType::ArgsMismatch(name, *params, args.len()),
                            self.location_from_exprs(args),
                        )));
                    }

                    return Ok(func(evaluated_args).unwrap_or_default());
                }
                NativeFunction::Loose(func) => {
                    return Ok(func(evaluated_args).unwrap_or_default());
                }
            }
        }

        let function = self
            .variables
            .iter()
            .find(|(ident, value)| matches!(value, Value::Function { .. }) && **ident == name);

        if let Some(function) = function {
            let function = function.1.clone();

            match function {
                Value::Function {
                    args: parameters,
                    body,
                } => {
                    if args.len() != parameters.len() {
                        return Err(Box::new(Error::new(
                            ErrorType::ArgsMismatch(name, parameters.len(), args.len()),
                            self.location_from_exprs(args),
                        )));
                    }

                    let scope = self.create_scope(body.clone());

                    for (param, arg) in parameters.iter().zip(evaluated_args) {
                        scope.add_variable(&param, arg);
                    }

                    scope.eval()
                }
                _ => unreachable!("`function` was filtered before to only match for functions"),
            }
        } else {
            Err(Box::new(Error::new(
                ErrorType::FunctionDoesntExist(name),
                None,
            )))
        }
    }
}
