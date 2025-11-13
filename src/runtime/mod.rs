use crate::{
    parser::{ast::types::Expr, parse},
    runtime::types::ValueResult,
};
use miette::NamedSource;
use std::{collections::HashMap, fmt::Debug, fs, rc::Rc};
pub use types::{Builtin, Error, ErrorKind, Value, ValueKind};

pub mod types;

#[cfg(feature = "serde")]
pub mod serde;

// Runtime Implementations
mod binary_op;
mod call;
mod expr;

#[derive(Debug)]
pub struct Scope {
    scopes: Vec<Scope>,
    variables: HashMap<String, Value>,

    ast: Rc<Expr>,
    source: NamedSource<String>,
}

impl Scope {
    #[allow(
        clippy::missing_panics_doc,
        reason = "The possible panic is checked beforehand"
    )]
    pub fn new(variables: HashMap<String, Value>, source: NamedSource<String>, ast: Expr) -> Self {
        Self {
            scopes: Vec::new(),
            variables,

            ast: Rc::new(ast),
            source,
        }
    }

    pub fn define(&mut self, name: impl ToString, value: impl Into<Value>) {
        self.variables.insert(name.to_string(), value.into());
    }

    /// Evaluates an AST expression.
    /// # Errors
    /// This function will return an error if an evaluation error occurs.
    pub fn eval(&mut self) -> ValueResult {
        #[allow(
            clippy::unwrap_used,
            reason = "The length of `args` is checked before by `eval_call`"
        )]
        {
            self.define(
                "if",
                Value::new_builtin(
                    Builtin(Rc::new(|ctx| {
                        let args_len = 3;

                        let cond = ctx.get_arg(0, args_len)?;
                        let then_branch = ctx.get_arg(1, args_len)?;
                        let else_branch = ctx.get_arg(2, args_len)?;

                        let mut scope = ctx.new_scope();

                        let cond = scope.eval_expr(&cond)?;

                        if cond.is_truthy() {
                            return scope.eval_expr(&then_branch);
                        }

                        scope.eval_expr(&else_branch)
                    }))
                    .into(),
                ),
            );
            self.define(
                "maybe",
                Value::new_builtin(
                    Builtin(Rc::new(|inputs| {
                        let cond = inputs.get_arg(0, 2)?;
                        let then = inputs.get_arg(1, 2)?;

                        let mut scope = Scope::new(inputs.variables, inputs.source, inputs.expr);

                        let cond = scope.eval_expr(&cond)?;

                        if cond.is_truthy() {
                            return Ok(cond);
                        }

                        scope.eval_expr(&then)
                    }))
                    .into(),
                ),
            );

            // let variables = self.variables.clone();
            // self.define(
            //     "map",
            //     Builtin::Strict {
            //         params: 2,
            //         func: Rc::new(move |args, _| {
            //             let Some(Value::Function {
            //                 args: callback_args,
            //                 expr: callback,
            //             }) = args.first()
            //             else {
            //                 return Err(Box::new(Error::new(
            //                     ErrorType::NativeFnError(
            //                         "`map` requires a function with a single argument as the first argument".to_string(),
            //                     ),
            //                     None,
            //                 )));
            //             };
            //             let Some(Value::Array(list)) = args.get(1) else {
            //                 return Err(Box::new(Error::new(
            //                     ErrorType::NativeFnError(
            //                         "`map` requires an array the second argument".to_string(),
            //                     ),
            //                     None,
            //                 )));
            //             };

            //             let mut new_list = Vec::with_capacity(list.len());

            //             if callback_args.len() != 1 {
            //                 return Err(Box::new(Error::new(
            //                     ErrorType::NativeFnError(
            //                         "`map` requires a function with a single argument as the first argument".to_string(),
            //                     ),
            //                     None,
            //                 )));
            //             }

            //             let Some(callback_arg) = callback_args.first() else {
            //                 unreachable!("length was checked before");
            //             };

            //             for item in list {
            //                 let mut scope = Scope::new(variables.clone(), Source::from_text(""), callback.clone());
            //                 scope.define(callback_arg.clone(), item.clone());
            //                 new_list.push(scope.eval()?);
            //             }

            //             Ok(Value::Array(new_list))
            //         }),
            //     },
            // );
            // self.define(
            //     "join",
            //     Builtin::Strict {
            //         params: 2,
            //         func: Rc::new(|args, _| {
            //             let Some(Value::Array(list)) = args.first() else {
            //                 return Err(Box::new(Error::new(
            //                     ErrorType::NativeFnError(
            //                         "`join` requires an array the first argument".to_string(),
            //                     ),
            //                     None,
            //                 )));
            //             };
            //             let Some(Value::String(sep)) = args.get(1) else {
            //                 return Err(Box::new(Error::new(
            //                     ErrorType::NativeFnError(
            //                         "`join` requires a separator string as the second argument"
            //                             .to_string(),
            //                     ),
            //                     None,
            //                 )));
            //             };

            //             #[allow(clippy::arithmetic_side_effects)]
            //             Ok(Value::String(
            //                 list.iter()
            //                     .map(Value::to_string)
            //                     .reduce(|a, b| a + sep + &b)
            //                     .unwrap_or_default(),
            //             ))
            //         }),
            //     },
            // );

            self.define(
                "import",
                Value::new_builtin(
                    Builtin(Rc::new(move |ctx| {
                        let (path, path_span) = {
                            let path = ctx.ensure_is_path(ctx.get_arg_evaluated(0, 1)?)?;
                            (path.data, path.span)
                        };

                        let file = fs::read_to_string(&path)
                            .map_err(|err| Error::new(err.into(), ctx.source.clone(), path_span))?;
                        let source = NamedSource::new(path.display().to_string(), file);
                        let ast = parse(&source).map_err(|err| {
                            let span = err.span;
                            let source = err.source.clone();
                            Error::new(err.into(), source, span)
                        })?;

                        Scope::new(ctx.variables, source, ast).eval()
                    }))
                    .into(),
                ),
            );

            //     #[cfg(feature = "fs")]
            //     self.define(
            //         "readFile",
            //         Builtin::Strict {
            //             params: 1,
            //             func: Rc::new(|args, _| {
            //                 let Some(Value::Path(path)) = args.first() else {
            //                     return Err(Box::new(Error::new(
            //                         ErrorType::NativeFnError(
            //                             "`readFile` requires a path as input".to_string(),
            //                         ),
            //                         None,
            //                     )));
            //                 };
            //                 let content =
            //                     fs::read_to_string(path).map_err(|err| Box::new(err.into()))?;

            //                 Ok(Value::String(content))
            //             }),
            //         },
            //     );

            //     #[cfg(feature = "fs")]
            //     self.define(
            //         "readDir",
            //         Builtin::Strict {
            //             params: 1,
            //             func: Rc::new(|args, _| {
            //                 use crate::object;

            //                 let Some(Value::Path(path)) = args.first() else {
            //                     return Err(Box::new(Error::new(
            //                         ErrorType::NativeFnError(
            //                             "`readDir` requires a path as input".to_string(),
            //                         ),
            //                         None,
            //                     )));
            //                 };
            //                 let content = fs::read_dir(path).map_err(|err| Box::new(err.into()))?;
            //                 let content = content
            //                     .into_iter()
            //                     .filter_map(Result::ok)
            //                     .map(|entry| {
            //                         object! {
            //                             path: Value::Path(entry.path()),
            //                             type: Value::String(
            //                                 match entry.file_type() {
            //                                     Ok(f) if f.is_file() => "file",
            //                                     Ok(f) if f.is_dir() => "dir",
            //                                     Ok(f) if f.is_symlink() => "symlink",
            //                                     _ => "other",
            //                                 }
            //                                 .into()
            //                             )
            //                         }
            //                     })
            //                     .collect::<Vec<_>>();

            //                 Ok(Value::Array(content))
            //             }),
            //         },
            //     );

            //     #[cfg(feature = "toml")]
            //     self.define(
            //         "toml",
            //         Builtin::Strict {
            //             params: 1,
            //             func: Rc::new(|args, _| {
            //                 fn convert_value(toml: toml::Value) -> ValueResult {
            //                     use std::collections::BTreeMap;

            //                     Ok(match toml {
            //                         toml::Value::String(v) => Value::String(v),
            //                         toml::Value::Integer(v) => {
            //                             Value::Int(v.try_into().map_err(|_| {
            //                                 Box::new(Error::new(
            //                                     ErrorType::NativeFnError(
            //                                         "Failed to convert integer while parsing toml file"
            //                                             .into(),
            //                                     ),
            //                                     None,
            //                                 ))
            //                             })?)
            //                         }
            //                         toml::Value::Float(v) => Value::Float(v),
            //                         toml::Value::Boolean(v) => Value::Boolean(v),
            //                         // TODO: This could probably be better.
            //                         toml::Value::Datetime(v) => Value::String(v.to_string()),
            //                         toml::Value::Array(v) => {
            //                             let mut values = Vec::new();

            //                             for toml_value in v {
            //                                 values.push(convert_value(toml_value)?);
            //                             }

            //                             Value::Array(values)
            //                         }
            //                         toml::Value::Table(v) => {
            //                             let mut object = BTreeMap::new();

            //                             for field in v {
            //                                 object.insert(field.0, convert_value(field.1)?);
            //                             }

            //                             Value::Object(object)
            //                         }
            //                     })
            //                 }

            //                 let Some(Value::String(content)) = args.first() else {
            //                     return Err(Box::new(Error::new(
            //                         ErrorType::NativeFnError(
            //                             "`toml` requires a toml string as an input".to_string(),
            //                         ),
            //                         None,
            //                     )));
            //                 };
            //                 let toml = toml::from_str::<toml::Value>(content)
            //                     .map_err(|err| Box::new(err.into()))?;

            //                 convert_value(toml)
            //             }),
            //         },
            //     );
            // }

            let ast_clone = Rc::clone(&self.ast);
            let value = self.eval_expr(&ast_clone)?;

            Ok(value)
        }
    }

    pub fn fetch_var(&self, name: &impl ToString) -> Option<&Value> {
        self.variables.get(&name.to_string())
    }

    #[allow(
        clippy::unwrap_used,
        clippy::missing_panics_doc,
        reason = "Value that is unwraped is inserted before in the same function."
    )]
    pub fn create_scope(&mut self, ast: Expr) -> &mut Scope {
        self.scopes
            .push(Scope::new(self.variables.clone(), self.source.clone(), ast));
        self.scopes.last_mut().unwrap()
    }
}
