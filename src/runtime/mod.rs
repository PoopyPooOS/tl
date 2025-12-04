use crate::{
    Source,
    parser::ast::types::Expr,
    runtime::{
        stdlib::stdlib,
        types::{NativeFn, value::ValueResult},
    },
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
mod stdlib;

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

impl Scope {
    pub fn new(variables: HashMap<String, Value>, source: Source, ast: Expr) -> Self {
        Self(Rc::new(ScopeInner {
            global_variables: Rc::new(RefCell::new(stdlib())),
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

    pub fn define_builtin(&self, key: impl Into<Value>, value: NativeFn) {
        self.0
            .global_variables
            .borrow_mut()
            .insert(key.into().to_string(), Builtin(value).into());
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
