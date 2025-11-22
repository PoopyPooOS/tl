use crate::{
    Source,
    parser::{ast::types::Expr, parse},
    runtime::types::value::ValueResult,
};
use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

pub use crate::runtime::types::{
    builtin::Builtin,
    error::{Error, ErrorKind},
    value::{Value, ValueKind},
};

pub mod types;

#[cfg(feature = "serde")]
pub mod serde;

// Runtime Implementations
mod binary_op;
mod call;
mod expr;

#[derive(Debug, Clone)]
struct ScopeInner {
    global_variables: Rc<RefCell<HashMap<String, Value>>>,
    local_variables: RefCell<HashMap<String, Value>>,

    parent: Option<Rc<ScopeInner>>,

    ast: Rc<Expr>,
    source: Rc<Source>,
}

#[derive(Debug, Clone)]
pub struct Scope(Rc<ScopeInner>);

pub fn get_global_builtins() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "if".to_owned(),
        Value::new_builtin(ValueKind::Builtin(Builtin(Rc::new(|ctx| {
            let args_len = 3;

            let cond = ctx.get_arg(0, args_len)?;
            let then_branch = ctx.get_arg(1, args_len)?;
            let else_branch = ctx.get_arg(2, args_len)?;

            let scope = ctx.new_scope();

            let cond = scope.eval_expr(&cond)?;

            if cond.is_truthy() {
                return scope.eval_expr(&then_branch);
            }

            scope.eval_expr(&else_branch)
        })))),
    );

    map.insert(
        "maybe".to_owned(),
        Value::new_builtin(ValueKind::Builtin(Builtin(Rc::new(|inputs| {
            let cond = inputs.get_arg_evaluated(0, 2)?;
            let then = inputs.get_arg(1, 2)?;

            if cond.is_truthy() {
                return Ok(cond);
            }

            let scope = inputs.new_scope();

            scope.eval_expr(&then)
        })))),
    );

    map.insert(
        "dbg".to_owned(),
        Value::new_builtin(ValueKind::Builtin(Builtin(Rc::new(|ctx| {
            let msg = ctx.get_arg_evaluated(0, 2)?;

            // TODO: Switch to `log` crate
            println!("{msg}");

            ctx.get_arg_evaluated(1, 2)
        })))),
    );

    map.insert(
        "import".to_owned(),
        Value::new_builtin(ValueKind::Builtin(Builtin(Rc::new(move |ctx| {
            let (path, path_span) = {
                let path = ctx.ensure_is_path(ctx.get_arg_evaluated(0, 1)?)?;
                (path.data, path.span)
            };

            let source = Source::path(path)
                .map_err(|err| Error::new(err.into(), (*ctx.source).clone(), path_span))?;

            let ast = parse(&source).map_err(|err| {
                let span = err.span;
                let source = err.source.clone();
                Error::new(err.into(), source, span)
            })?;

            Scope::new(HashMap::new(), source, ast).eval()
        })))),
    );

    map
}

impl Scope {
    pub fn new(variables: HashMap<String, Value>, source: Source, ast: Expr) -> Self {
        let builtins = get_global_builtins();
        Self(Rc::new(ScopeInner {
            global_variables: Rc::new(RefCell::new(builtins)),
            parent: None,
            local_variables: RefCell::new(variables),
            ast: Rc::new(ast),
            source: Rc::new(source),
        }))
    }

    pub fn define(&self, name: impl ToString, value: impl Into<Value>) {
        self.0
            .local_variables
            .borrow_mut()
            .insert(name.to_string(), value.into());
    }

    pub fn define_global(&self, name: impl ToString, value: impl Into<Value>) {
        self.0
            .global_variables
            .borrow_mut()
            .insert(name.to_string(), value.into());
    }

    /// Evaluates an AST expression.
    /// # Errors
    /// This function will return an error if an evaluation error occurs.
    pub fn eval(&self) -> ValueResult {
        let value = self.eval_expr(&Rc::clone(&self.0.ast))?;
        Ok(value)
    }

    pub fn fetch_var(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.0.global_variables.borrow().get(name) {
            return Some(v.clone());
        }

        if let Some(v) = self.0.local_variables.borrow().get(name) {
            return Some(v.clone());
        }

        self.0
            .parent
            .as_ref()
            .and_then(|p| Scope(Rc::clone(p)).fetch_var(name))
    }

    pub fn create_scope(&self, ast: Expr) -> Scope {
        Scope(Rc::new(ScopeInner {
            global_variables: Rc::clone(&self.0.global_variables),
            parent: Some(Rc::clone(&self.0)),
            local_variables: RefCell::new(HashMap::new()),
            ast: Rc::new(ast),
            source: Rc::clone(&self.0.source),
        }))
    }
}
