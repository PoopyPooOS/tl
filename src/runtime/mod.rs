use crate::{
    parser::ast::types::{Expr, Statement, StatementType},
    Source,
};
use logger::Location;
use std::{collections::HashMap, fmt::Debug, rc::Rc};
use types::{Error, NativeFunction, Value};

pub mod types;

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    #[must_use]
    pub fn new(source: Source, ast: Vec<Statement>) -> Self {
        Self {
            ast: Rc::new(ast),
            source,
            ..Default::default()
        }
    }

    pub fn add_variable(&mut self, name: &impl ToString, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// Evaluates a list of statements (an AST).
    /// # Errors
    /// This function will return an error if an evaluation error occurs.
    #[allow(
        clippy::missing_panics_doc,
        reason = "Clippy bug as the panic is in a closure"
    )]
    pub fn eval(&mut self) -> ValueResult {
        let mut value = None;

        #[allow(
            clippy::unwrap_used,
            reason = "The length of `args` is checked before by `eval_call`"
        )]
        {
            self.native_functions.insert(
                "if".into(),
                NativeFunction::Strict {
                    params: 3,
                    func: Box::new(|args| {
                        let cond = args.first().unwrap();
                        let then_branch = args.get(1).unwrap();
                        let else_branch = args.get(2).unwrap();

                        if cond.is_truthy() {
                            Some(then_branch.clone())
                        } else {
                            Some(else_branch.clone())
                        }
                    }),
                },
            );

            self.native_functions.insert(
                "println".into(),
                NativeFunction::Loose(Box::new(|args| {
                    for arg in args {
                        println!("{arg}");
                    }
                    None
                })),
            );
            self.native_functions.insert(
                "print".into(),
                NativeFunction::Loose(Box::new(|args| {
                    for arg in args {
                        print!("{arg}");
                    }
                    None
                })),
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

    pub(crate) fn fetch_var(&self, name: &impl ToString) -> Option<&Value> {
        self.variables.get(&name.to_string())
    }

    #[allow(
        clippy::unwrap_used,
        reason = "Value that is unwraped is inserted before in the same function."
    )]
    pub(crate) fn create_scope(&mut self, ast: Vec<Statement>) -> &mut Scope {
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
    pub(crate) fn location_from_expr(&self, expr: &Expr) -> Option<Location> {
        Some(Location {
            path: self.source.path.clone(),
            text: self.source.text.clone(),
            lines: expr.line..=expr.line,
            section: Some(expr.cols.clone()),
        })
    }

    /// Returns `None` if `args` is empty.
    fn location_from_exprs(&self, args: &[Expr]) -> Option<Location> {
        let mut line_start: Option<usize> = None;
        let mut line_end: Option<usize> = None;
        let mut start: Option<usize> = None;
        let mut end: Option<usize> = None;

        if let Some(first) = args.first() {
            line_start = Some(first.line);
            start = Some(*first.cols.start());
        }

        if let Some(last) = args.last() {
            line_end = Some(last.line);
            end = Some(*last.cols.end());
        }

        if [line_start, line_end, start, end]
            .iter()
            .all(Option::is_some)
        {
            #[allow(clippy::unwrap_used, reason = "Checked before")]
            Some(Location {
                path: self.source.path.clone(),
                text: self.source.text.clone(),
                lines: line_start.unwrap()..=line_end.unwrap(),
                section: Some(start.unwrap()..=end.unwrap()),
            })
        } else {
            None
        }
    }
}
