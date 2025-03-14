use crate::{
    Source,
    parser::{
        ast::types::{Expr, Statement, StatementType},
        parse,
    },
};
use logger::Location;
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    fs,
    path::PathBuf,
    rc::Rc,
};
use types::{Error, ErrorType, NativeFunction, Value};

pub mod types;

#[cfg(feature = "serde")]
pub mod serde;

// Runtime Implementations
mod binary_op;
mod call;
mod expr;

#[derive(Default)]
pub struct Scope {
    variables: HashMap<String, Value>,
    native_functions: HashMap<String, NativeFunction>,
    scopes: HashMap<usize, Scope>,
    ast: Rc<Vec<Statement>>,

    source: Source,
}

impl Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scope")
            .field("variables", &self.variables)
            .field("native_functions", &self.native_functions.keys())
            .field("scopes", &self.scopes)
            .field("ast", &self.ast)
            .field("source", &self.source)
            .finish()
    }
}

type ValueResult = Result<Value, Box<Error>>;

impl Scope {
    pub fn new(source: Source, ast: Vec<Statement>) -> Self {
        if cfg!(debug_assertions) {
            logger::set_app_name!("Runtime");
        }

        Self {
            ast: Rc::new(ast),
            source,
            ..Default::default()
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn add_variable(&mut self, name: impl ToString, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn add_native_fn(&mut self, name: impl ToString, native_fn: NativeFunction) {
        self.native_functions.insert(name.to_string(), native_fn);
    }

    /// Evaluates a list of statements (an AST).
    /// # Errors
    /// This function will return an error if an evaluation error occurs.
    #[allow(clippy::missing_panics_doc)]
    pub fn eval(&mut self) -> ValueResult {
        let mut value = None;

        #[allow(
            clippy::unwrap_used,
            reason = "The length of `args` is checked before by `eval_call`"
        )]
        {
            self.native_functions.insert(
                "println".into(),
                NativeFunction::Loose(Box::new(|args| {
                    for arg in args {
                        println!("{arg}");
                    }
                    Ok(Value::Null)
                })),
            );
            self.native_functions.insert(
                "print".into(),
                NativeFunction::Loose(Box::new(|args| {
                    for arg in args {
                        print!("{arg}");
                    }
                    Ok(Value::Null)
                })),
            );

            self.add_native_fn(
                "import",
                NativeFunction::Strict {
                    params: 1,
                    func: Box::new(|args| {
                        let Value::String(path) = args.first().unwrap() else {
                            return Err(Box::new(Error::new(
                                ErrorType::NativeFnError(
                                    "`import` requires a path string as an input".to_string(),
                                ),
                                None,
                            )));
                        };
                        let path = PathBuf::from(path);
                        let source = Source::from_path(path).map_err(Error::from)?;
                        let ast = parse(&source).map_err(|err| Error::from(*err))?;

                        Scope::new(source, ast).eval()
                    }),
                },
            );

            #[cfg(feature = "fs")]
            self.add_native_fn(
                "readFile",
                NativeFunction::Strict {
                    params: 1,
                    func: Box::new(|args| {
                        let Value::String(path) = args.first().unwrap() else {
                            return Err(Box::new(Error::new(
                                ErrorType::NativeFnError(
                                    "`readFile` requires a path string as an input".to_string(),
                                ),
                                None,
                            )));
                        };
                        let path = PathBuf::from(path);
                        let content =
                            fs::read_to_string(path).map_err(|err| Box::new(err.into()))?;

                        Ok(Value::String(content))
                    }),
                },
            );

            #[cfg(feature = "toml")]
            self.add_native_fn(
                "toml",
                NativeFunction::Strict {
                    params: 1,
                    func: Box::new(|args| {
                        let Value::String(content) = args.first().unwrap().clone() else {
                            return Err(Box::new(Error::new(
                                ErrorType::NativeFnError(
                                    "`toml` requires a toml string as an input".to_string(),
                                ),
                                None,
                            )));
                        };
                        let toml = toml::from_str::<toml::Value>(&content)
                            .map_err(|err| Box::new(err.into()))?;

                        #[allow(clippy::items_after_statements)]
                        fn convert_value(toml: toml::Value) -> ValueResult {
                            use std::collections::BTreeMap;

                            Ok(match toml {
                                toml::Value::String(v) => Value::String(v),
                                toml::Value::Integer(v) => {
                                    Value::Int(v.try_into().map_err(|_| {
                                        Box::new(Error::new(
                                            ErrorType::NativeFnError(
                                                "Failed to convert integer while parsing toml file"
                                                    .into(),
                                            ),
                                            None,
                                        ))
                                    })?)
                                }
                                toml::Value::Float(v) => Value::Float(v),
                                toml::Value::Boolean(v) => Value::Boolean(v),
                                // TODO: This could probably be better.
                                toml::Value::Datetime(v) => Value::String(v.to_string()),
                                toml::Value::Array(v) => {
                                    let mut values = Vec::new();

                                    for toml_value in v {
                                        values.push(convert_value(toml_value)?);
                                    }

                                    Value::Array(values)
                                }
                                toml::Value::Table(v) => {
                                    let mut object = BTreeMap::new();

                                    for field in v {
                                        object.insert(field.0, convert_value(field.1)?);
                                    }

                                    Value::Object(object)
                                }
                            })
                        }

                        convert_value(toml)
                    }),
                },
            );
        }

        let ast_clone = Rc::clone(&self.ast);
        for node in ast_clone.iter() {
            value = match &node.statement_type {
                StatementType::Expr(expr) => Some(self.eval_expr(expr)?),
                StatementType::Let { name, value } => {
                    let value = self.eval_expr(value)?;
                    self.add_variable(name, value);
                    None
                }
            };
        }

        Ok(value.unwrap_or_default())
    }

    pub fn fetch_var(&self, name: &impl ToString) -> Option<&Value> {
        self.variables.get(&name.to_string())
    }

    #[allow(
        clippy::unwrap_used,
        clippy::missing_panics_doc,
        reason = "Value that is unwraped is inserted before in the same function."
    )]
    pub fn create_scope(&mut self, ast: Vec<Statement>) -> &mut Scope {
        let scope_id = self.scopes.len();
        self.scopes
            .insert(scope_id, Scope::new(self.source.clone(), ast));
        self.scopes.get_mut(&scope_id).unwrap()
    }

    #[allow(
        clippy::unnecessary_wraps,
        reason = "`Option<Location>` is more commonly used, this simplifies things"
    )]
    /// Always returns `Some`, safe to unwrap if needed.
    fn location_from_expr(&self, expr: &Expr) -> Option<Location> {
        Some(Location {
            path: self.source.path.clone(),
            text: self.source.text.clone(),
            section: Some(expr.section.clone()),
        })
    }
}
