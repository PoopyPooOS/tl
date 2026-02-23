#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

use crate::{
    parser::ast::types::expr::Expr,
    runtime::{Builtin, Error, Scope, types::Function},
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
    pub metadata: Box<Metadata>,
}

#[derive(Debug, Default, Clone)]
pub struct Metadata {
    pub doc: Option<String>,
}

impl Default for Value {
    fn default() -> Self {
        Self::new_builtin(ValueKind::Null)
    }
}

impl Value {
    pub fn new(kind: ValueKind, span: SourceSpan) -> Self {
        Self {
            kind,
            span,
            metadata: Box::new(Metadata::default()),
        }
    }

    pub fn new_builtin(kind: ValueKind) -> Self {
        Self::new(kind, SourceSpan::new(0.into(), 0))
    }
}

#[derive(Default, Clone)]
pub enum ValueKind {
    #[default]
    Null,
    Bool(bool),
    Int(isize),
    Float(f64),
    String(String),
    Path(PathBuf),
    Array(Vec<Value>),
    Object(IndexMap<String, Value>),
    Function(Function),
    Builtin(Builtin),
    Thunk {
        def_scope: Box<Scope>,
        expr: Expr,
    },
}

impl Debug for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueKind::Null => f.debug_tuple("Null").finish(),
            ValueKind::Bool(v) => f.debug_tuple("Boolean").field(v).finish(),
            ValueKind::Int(v) => f.debug_tuple("Int").field(v).finish(),
            ValueKind::Float(v) => f.debug_tuple("Float").field(v).finish(),
            ValueKind::String(v) => f.debug_tuple("String").field(v).finish(),
            ValueKind::Path(v) => f.debug_tuple("Path").field(v).finish(),
            ValueKind::Array(v) => f.debug_tuple("Array").field(v).finish(),
            ValueKind::Object(v) => f.debug_tuple("Object").field(v).finish(),
            ValueKind::Function(f_val) => {
                f // Dont include `closure_scope` here, it would loop forever
                    .debug_struct("Function")
                    .field("params", &f_val.params)
                    .field("return_type", &f_val.return_type)
                    .field("body", &f_val.body)
                    .finish_non_exhaustive()
            }
            ValueKind::Builtin(v) => f.debug_tuple("Builtin").field(v).finish(),
            ValueKind::Thunk { expr, .. } => f
                .debug_struct("Thunk")
                .field("expr", expr)
                .finish_non_exhaustive(),
        }
    }
}

impl ValueKind {
    pub fn type_of(&self) -> &'static str {
        match &self {
            ValueKind::Null => "null",
            ValueKind::Bool(_) => "bool",
            ValueKind::Int(_) => "number",
            ValueKind::Float(_) => "float",
            ValueKind::String(_) => "string",
            ValueKind::Path(_) => "path",
            ValueKind::Array(_) => "array",
            ValueKind::Object(_) => "object",
            ValueKind::Function(_) => "function",
            ValueKind::Builtin(..) => "builtin",
            ValueKind::Thunk { .. } => "thunk",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match &self {
            ValueKind::Bool(b) => *b,
            ValueKind::Int(n) => *n > 0,
            ValueKind::Float(f) => *f > 0.0,
            ValueKind::String(s) => !s.is_empty(),
            ValueKind::Path(p) => !p.exists(),
            ValueKind::Array(arr) => !arr.is_empty(),
            ValueKind::Object(map) => !map.is_empty(),
            ValueKind::Function(_)
            | ValueKind::Builtin(..)
            | ValueKind::Thunk { .. }
            | ValueKind::Null => false,
        }
    }

    pub fn is_callable(&self) -> bool {
        matches!(self, ValueKind::Function(_) | ValueKind::Builtin(..))
    }

    pub fn and(&self, rhs: &Value) -> bool {
        self.is_truthy() && rhs.is_truthy()
    }

    pub fn or(&self, rhs: &Value) -> bool {
        self.is_truthy() || rhs.is_truthy()
    }

    /// Helper function to create thunks with an empty scope as they don't get accounted for in the `PartialEq` impl.
    #[cfg(test)]
    pub fn thunk(expr: Expr) -> Self {
        use crate::Source;
        use std::collections::HashMap;

        Self::Thunk {
            def_scope: Box::new(Scope::new(
                HashMap::new(),
                Source::text(""),
                Expr::default(),
            )),
            expr,
        }
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
                    metadata: Box::new(Metadata::default()),
                })
                .clone(),
            _ => Value {
                kind: ValueKind::Null,
                span: self.span,
                metadata: Box::new(Metadata::default()),
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
            ValueKind::Bool(v) => f.write_str(v.to_string().as_str()),
            ValueKind::Int(v) => f.write_str(v.to_string().as_str()),
            ValueKind::Float(v) => f.write_str(v.to_string().as_str()),
            ValueKind::String(v) => f.write_str(v),
            ValueKind::Path(v) => f.write_str(&v.display().to_string()),
            ValueKind::Array(v) => {
                let formatted = v.iter().map(ToString::to_string).collect::<Vec<_>>();
                f.write_str("[\n  ")?;
                f.write_str(&format!("{}\n]", formatted.join("\n  ")))
            }
            ValueKind::Object(v) => {
                let formatted = v
                    .iter()
                    .map(|(k, v)| format!("{k} = {v}"))
                    .collect::<Vec<_>>();
                f.write_str("{\n  ")?;
                f.write_str(&format!("{}\n}}", formatted.join("\n  ")))
            }
            ValueKind::Function(_) => f.write_str("function"),
            ValueKind::Builtin { .. } => f.write_str("builtin"),
            ValueKind::Thunk { .. } => f.write_str("<thunk>"),
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
