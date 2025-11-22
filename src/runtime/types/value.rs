#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

use crate::{
    parser::ast::types::expr::Expr,
    runtime::{Builtin, Error, Scope},
};
use indexmap::IndexMap;
use miette::SourceSpan;
use std::{
    fmt::{self, Debug, Display},
    ops::Index,
    path::PathBuf,
};

pub type ValueResult = Result<Value, Error>;

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub span: SourceSpan,
}

impl Default for Value {
    fn default() -> Self {
        Self::new_builtin(ValueKind::Null)
    }
}

impl Value {
    pub const fn new(kind: ValueKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub fn new_builtin(kind: ValueKind) -> Self {
        Self::new(kind, SourceSpan::new(0.into(), 0))
    }
}

#[derive(Debug, Default, Clone)]
pub enum ValueKind {
    #[default]
    Null,
    Boolean(bool),
    Int(isize),
    Float(f64),
    String(String),
    Path(PathBuf),
    Array(Vec<Value>),
    Object(IndexMap<String, Value>),
    Function {
        def_scope: Box<Scope>,
        arg: String,
        expr: Expr,
    },
    Builtin(Builtin),
}

impl ValueKind {
    pub fn type_of(&self) -> &'static str {
        match &self {
            ValueKind::Null => "null",
            ValueKind::Boolean(_) => "boolean",
            ValueKind::Int(_) => "number",
            ValueKind::Float(_) => "float",
            ValueKind::String(_) => "string",
            ValueKind::Path(_) => "path",
            ValueKind::Array(_) => "array",
            ValueKind::Object(_) => "object",
            ValueKind::Function { .. } => "function",
            ValueKind::Builtin(..) => "builtin",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match &self {
            ValueKind::Boolean(b) => *b,
            ValueKind::Int(n) => *n > 0,
            ValueKind::Float(f) => *f > 0.0,
            ValueKind::String(s) => !s.is_empty(),
            ValueKind::Path(p) => !p.exists(),
            ValueKind::Array(arr) => !arr.is_empty(),
            ValueKind::Object(map) => !map.is_empty(),
            ValueKind::Function { .. } | ValueKind::Builtin(..) | ValueKind::Null => false,
        }
    }

    pub fn is_callable(&self) -> bool {
        matches!(self, ValueKind::Function { .. } | ValueKind::Builtin(..))
    }

    pub fn and(&self, rhs: &Value) -> bool {
        self.is_truthy() && rhs.is_truthy()
    }

    pub fn or(&self, rhs: &Value) -> bool {
        self.is_truthy() || rhs.is_truthy()
    }
}

impl Value {
    pub fn type_of(&self) -> &'static str {
        self.kind.type_of()
    }

    pub fn is_truthy(&self) -> bool {
        self.kind.is_truthy()
    }

    pub fn is_callable(&self) -> bool {
        self.kind.is_callable()
    }

    pub fn and(&self, rhs: &Self) -> bool {
        self.kind.and(rhs)
    }

    pub fn or(&self, rhs: &Self) -> bool {
        self.kind.or(rhs)
    }

    #[must_use]
    pub fn access(&self, rhs: impl Into<String>) -> Self {
        match &self.kind {
            ValueKind::Object(v) => v
                .get(&rhs.into())
                .unwrap_or(&Value {
                    kind: ValueKind::Null,
                    span: self.span,
                })
                .clone(),
            _ => Value {
                kind: ValueKind::Null,
                span: self.span,
            },
        }
    }

    pub fn try_index(&self, index: usize) -> Result<&Self, usize> {
        match &self.kind {
            ValueKind::Array(v) => v.get(index).ok_or(v.len()),
            _ => Err(0),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ValueKind::Null => f.write_str("null"),
            ValueKind::Boolean(v) => f.write_str(v.to_string().as_str()),
            ValueKind::Int(v) => f.write_str(v.to_string().as_str()),
            ValueKind::Float(v) => f.write_str(v.to_string().as_str()),
            ValueKind::String(v) => f.write_str(v),
            ValueKind::Path(v) => f.write_str(&v.display().to_string()),
            ValueKind::Array(v) => {
                let formatted = v.iter().map(ToString::to_string).collect::<Vec<_>>();
                f.write_str(&format!("[ {} ]", formatted.join(" ")))
            }
            ValueKind::Object(v) => {
                let formatted = v
                    .iter()
                    .map(|(k, v)| format!("{k} = {v}"))
                    .collect::<Vec<_>>();
                f.write_str(&format!("{{ {} }}", formatted.join("; ")))
            }
            ValueKind::Function { .. } => f.write_str("function"),
            ValueKind::Builtin { .. } => f.write_str("builtin"),
        }
    }
}

impl Index<usize> for ValueKind {
    type Output = Self;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::Array(v) => v.get(index).map_or(&Self::Null, |v| &v.kind),
            _ => &Self::Null,
        }
    }
}
