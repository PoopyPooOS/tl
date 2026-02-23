use crate::{
    Source,
    parser::ast::types::Expr,
    runtime::{
        extension::Registry,
        stdlib::stdlib,
        types::{function::Builtin, value::ValueResult},
    },
};
use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

pub use crate::runtime::types::{
    error::{Error, ErrorKind},
    value::{Value, ValueKind},
};

pub mod extension;
pub mod serde;
pub mod types;

// Runtime Implementations
mod binary_op;
mod call;
mod expr;
mod stdlib;
mod r#type;

#[derive(Debug, Clone)]
pub struct ScopeInner {
    global_variables: Rc<RefCell<HashMap<String, Value>>>,
    local_variables: RefCell<HashMap<String, Value>>,

    parent: Option<Rc<ScopeInner>>,

    ast: Rc<Expr>,
    source: Rc<Source>,
}

#[derive(Debug, Clone)]
pub struct Scope(pub Rc<ScopeInner>);

impl Scope {
    pub fn new(variables: HashMap<String, Value>, source: Source, ast: Expr) -> Self {
        let mut global_variables = stdlib();

        let registry = Registry::read();

        if let Some(Registry(registry)) = registry {
            let extensions = registry
                .into_iter()
                .map(|(name, entry)| (name, Value::new_builtin(ValueKind::Path(entry.path))))
                .collect();

            global_variables.insert(
                "ext".to_owned(),
                Value::new_builtin(ValueKind::Object(extensions)),
            );
        }

        Self(Rc::new(ScopeInner {
            global_variables: Rc::new(RefCell::new(global_variables)),
            parent: None,
            local_variables: RefCell::new(variables),
            ast: Rc::new(ast),
            source: Rc::new(source),
        }))
    }

    pub fn define(&self, key: impl Into<Value>, value: impl Into<Value>) {
        self.0
            .local_variables
            .borrow_mut()
            .insert(key.into().to_string(), value.into());
    }

    pub fn define_global(&self, key: impl Into<Value>, value: impl Into<Value>) {
        self.0
            .global_variables
            .borrow_mut()
            .insert(key.into().to_string(), value.into());
    }

    pub fn define_builtin(&self, key: impl Into<Value>, value: Builtin) {
        self.0.global_variables.borrow_mut().insert(
            key.into().to_string(),
            Value::new_builtin(ValueKind::Builtin(value)),
        );
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

    pub fn force(&self, value: Value) -> ValueResult {
        match value.kind {
            ValueKind::Thunk { def_scope, expr } => def_scope.eval_expr(&expr),
            _ => Ok(value),
        }
    }
}
