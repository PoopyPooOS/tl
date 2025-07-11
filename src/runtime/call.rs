use super::{
    types::{Error, ErrorType, NativeFunction, Value},
    ValueResult,
};
use crate::parser::ast::types::{Expr, ExprType};

impl super::Scope {
    pub(super) fn eval_call(&mut self, expr: &Expr) -> ValueResult {
        let ExprType::Call { name, args } = &expr.expr_type else {
            unreachable!()
        };

        let name = name.to_string();

        #[allow(clippy::unwrap_used, clippy::single_match)]
        match name.as_str() {
            "if" => {
                let cond = args.first().ok_or_else(|| {
                    Error::new(
                        ErrorType::ArgsMismatch("if".into(), 3, args.len()),
                        self.location_from_expr(expr),
                    )
                })?;
                let then_branch = args.get(1).ok_or_else(|| {
                    Error::new(
                        ErrorType::ArgsMismatch("if".into(), 3, args.len()),
                        self.location_from_expr(expr),
                    )
                })?;
                let else_branch = args.get(2).ok_or_else(|| {
                    Error::new(
                        ErrorType::ArgsMismatch("if".into(), 3, args.len()),
                        self.location_from_expr(expr),
                    )
                })?;

                let cond = self.eval_expr(cond)?;

                if cond.is_truthy() {
                    return self.eval_expr(then_branch);
                }

                return self.eval_expr(else_branch);
            }
            "maybe" => {
                let cond = args.first().ok_or_else(|| {
                    Error::new(
                        ErrorType::ArgsMismatch("if".into(), 3, args.len()),
                        self.location_from_expr(expr),
                    )
                })?;
                let then = args.get(1).ok_or_else(|| {
                    Error::new(
                        ErrorType::ArgsMismatch("if".into(), 3, args.len()),
                        self.location_from_expr(expr),
                    )
                })?;

                let cond = self.eval_expr(cond)?;

                if cond.is_truthy() {
                    return Ok(cond);
                }

                return self.eval_expr(then);
            }
            _ => (),
        }

        let mut evaluated_args = Vec::new();
        for expr in args {
            evaluated_args.push(self.eval_expr(expr)?);
        }
        let evaluated_args = evaluated_args;

        if let Some(native_fn) = self.native_functions.get(&name) {
            let mut result = match native_fn {
                NativeFunction::Strict { params, func } => {
                    if args.len() != *params {
                        return Err(Box::new(Error::new(
                            ErrorType::ArgsMismatch(name, *params, args.len()),
                            self.location_from_expr(expr),
                        )));
                    }

                    func(evaluated_args)
                }
                NativeFunction::Loose(func) => func(evaluated_args),
            };

            // Set error location here because native functions will almost always just return `None` for the location.
            if let Err(ref mut err) = result
                && matches!(err.error_type, ErrorType::NativeFnError(_))
                && err.location.is_none()
            {
                err.location = self.location_from_expr(expr);
            }

            return result;
        }

        let function = self
            .variables
            .iter()
            .find(|(ident, value)| matches!(value, Value::Function { .. }) && **ident == name);

        if let Some(function) = function {
            let function = function.1.clone();

            match function {
                Value::Function {
                    args: ref parameters,
                    ref body,
                } => {
                    if args.len() != parameters.len() {
                        return Err(Box::new(Error::new(
                            ErrorType::ArgsMismatch(name, parameters.len(), args.len()),
                            self.location_from_expr(expr),
                        )));
                    }

                    let scope = self.create_scope(body.clone());

                    // Add arguments into scope
                    for (param, arg) in parameters.iter().zip(evaluated_args) {
                        scope.add_variable(param, arg);
                    }

                    // Add the function itself into scope
                    scope.add_variable(&name, function);

                    scope.eval()
                }
                _ => unreachable!("`function` was filtered before to only match for functions"),
            }
        } else {
            Err(Box::new(Error::new(
                ErrorType::FunctionDoesntExist(name),
                self.location_from_expr(expr),
            )))
        }
    }
}
