#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

use crate::{
    merge_spans,
    parser::ast::{
        self,
        types::{Expr, ExprKind},
    },
    runtime::Scope,
};
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    fmt::{self, Debug, Display},
    io,
    ops::{Add, Div, Index, Mul, Rem, Sub},
    path::PathBuf,
    rc::Rc,
};
use thiserror::Error;

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
    Object(BTreeMap<String, Value>),
    Function {
        args: Vec<String>,
        expr: Expr,
    },
    Builtin(Builtin),
}

#[derive(Debug, Clone)]
pub struct ExtractedValue<T> {
    pub data: T,
    pub span: SourceSpan,
}

#[derive(Clone)]
pub struct Builtin(pub NativeFn);

pub type NativeFn = Rc<dyn Fn(NativeFnCtx) -> ValueResult>;

pub struct NativeFnCtx {
    pub expr: Expr,
    pub variables: HashMap<String, Value>,
    pub source: NamedSource<String>,
}

impl NativeFnCtx {
    pub fn new_scope(&self) -> Scope {
        Scope::new(
            self.variables.clone(),
            self.source.clone(),
            self.expr.clone(),
        )
    }

    pub fn get_arg(&self, index: usize, expected_len: usize) -> Result<Expr, Error> {
        let ExprKind::Call { args, .. } = &self.expr.kind else {
            unreachable!()
        };

        let arg = args.get(index).ok_or(Error::new(
            ErrorKind::ArgsMismatch {
                len: expected_len,
                args: self.call_args_span(),
            },
            self.source.clone(),
            self.expr.span,
        ))?;

        Ok(arg.clone())
    }

    pub fn get_arg_evaluated(&self, index: usize, expected_len: usize) -> ValueResult {
        let ExprKind::Call { args, .. } = &self.expr.kind else {
            unreachable!()
        };

        let arg = args.get(index).ok_or(Error::new(
            ErrorKind::ArgsMismatch {
                len: expected_len,
                args: self.call_args_span(),
            },
            self.source.clone(),
            self.expr.span,
        ))?;

        self.eval_expr(arg.clone())
    }

    pub fn eval_expr(&self, expr: Expr) -> ValueResult {
        let mut scope = Scope::new(
            self.variables.clone(),
            self.source.clone(),
            self.expr.clone(),
        );

        scope.eval_expr(&expr)
    }

    pub fn expr_args(&self) -> Vec<Expr> {
        let ExprKind::Call { ref args, .. } = self.expr.kind else {
            unreachable!()
        };

        args.clone()
    }

    pub fn expr_args_evaluated(&self) -> Vec<ValueResult> {
        let mut scope = Scope::new(
            self.variables.clone(),
            self.source.clone(),
            self.expr.clone(),
        );

        let args = self.expr_args();

        args.iter().map(|arg| scope.eval_expr(arg)).collect()
    }

    pub fn call_args_span(&self) -> SourceSpan {
        let ExprKind::Call { ref args, .. } = self.expr.kind else {
            unreachable!()
        };

        let mut args_spans = args.iter().map(|arg| arg.span);

        if let Some(start) = args_spans.next()
            && let Some(end) = args_spans.next_back()
        {
            SourceSpan::new(start.offset().into(), end.len())
        } else {
            self.expr.span
        }
    }

    pub fn ensure_is_null(&self, value: Value) -> ValueResult {
        match value.kind {
            ValueKind::Null => Ok(value),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: ValueKind::Null.type_of().to_string(),
                    got: value.type_of().into(),
                },
                self.source.clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_boolean(&self, value: Value) -> Result<ExtractedValue<bool>, Error> {
        match value.kind {
            ValueKind::Boolean(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: ValueKind::Boolean(false).type_of().to_string(),
                    got: value.type_of().into(),
                },
                self.source.clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_int(&self, value: Value) -> Result<ExtractedValue<isize>, Error> {
        match value.kind {
            ValueKind::Int(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: ValueKind::Int(0).type_of().to_string(),
                    got: value.type_of().into(),
                },
                self.source.clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_float(&self, value: Value) -> Result<ExtractedValue<f64>, Error> {
        match value.kind {
            ValueKind::Float(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: ValueKind::Float(0.0).type_of().to_string(),
                    got: value.type_of().into(),
                },
                self.source.clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_string(&self, value: Value) -> Result<ExtractedValue<String>, Error> {
        match value.kind {
            ValueKind::String(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: ValueKind::String(String::new()).type_of().to_string(),
                    got: value.type_of().into(),
                },
                self.source.clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_path(&self, value: Value) -> Result<ExtractedValue<PathBuf>, Error> {
        match value.kind {
            ValueKind::Path(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: ValueKind::Path(PathBuf::new()).type_of().to_string(),
                    got: value.type_of().into(),
                },
                self.source.clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_array(&self, value: Value) -> Result<ExtractedValue<Vec<Value>>, Error> {
        match value.kind {
            ValueKind::Array(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: ValueKind::Array(Vec::new()).type_of().to_string(),
                    got: value.type_of().into(),
                },
                self.source.clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_object(
        &self,
        value: Value,
    ) -> Result<ExtractedValue<BTreeMap<String, Value>>, Error> {
        match value.kind {
            ValueKind::Object(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: ValueKind::Object(BTreeMap::new()).type_of().to_string(),
                    got: value.type_of().into(),
                },
                self.source.clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_function(
        &self,
        value: Value,
    ) -> Result<ExtractedValue<(Vec<String>, Expr)>, Error> {
        match value.kind {
            ValueKind::Function { args, expr } => Ok(ExtractedValue {
                data: (args, expr),
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: ValueKind::Function {
                        args: Vec::new(),
                        expr: Expr::default(),
                    }
                    .type_of()
                    .to_string(),
                    got: value.type_of().into(),
                },
                self.source.clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_builtin(&self, value: Value) -> Result<ExtractedValue<Builtin>, Error> {
        match value.kind {
            ValueKind::Builtin(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: ValueKind::Builtin(Builtin(Rc::new(|_| Ok(Value::default()))))
                        .type_of()
                        .to_string(),
                    got: value.type_of().into(),
                },
                self.source.clone(),
                self.expr.span,
            )),
        }
    }
}

impl Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Builtin")
            .field(&"<native function>")
            .finish()
    }
}

impl From<()> for ValueKind {
    fn from((): ()) -> Self {
        Self::Null
    }
}

impl From<bool> for ValueKind {
    fn from(val: bool) -> Self {
        Self::Boolean(val)
    }
}

impl From<isize> for ValueKind {
    fn from(val: isize) -> Self {
        Self::Int(val)
    }
}

impl From<f64> for ValueKind {
    fn from(val: f64) -> Self {
        Self::Float(val)
    }
}

impl<'a> From<&'a str> for ValueKind {
    fn from(val: &'a str) -> Self {
        Self::String(val.into())
    }
}

impl From<String> for ValueKind {
    fn from(val: String) -> Self {
        Self::String(val)
    }
}

impl From<PathBuf> for ValueKind {
    fn from(val: PathBuf) -> Self {
        Self::Path(val)
    }
}

impl<T: Into<Value>> From<Vec<T>> for ValueKind {
    fn from(val: Vec<T>) -> Self {
        Self::Array(val.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>> From<BTreeMap<String, T>> for ValueKind {
    fn from(val: BTreeMap<String, T>) -> Self {
        Self::Object(val.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl From<Builtin> for ValueKind {
    fn from(val: Builtin) -> Self {
        Self::Builtin(val)
    }
}

#[macro_export]
macro_rules! object {
    ($($key:ident: $val:expr),* $(,)?) => {
        Value::new_builtin($crate::runtime::ValueKind::Object(std::collections::BTreeMap::from([
            $(
                (stringify!($key).to_owned(), Value::new_builtin($val.into())),
            )*
        ])))
    };
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

impl Add for ValueKind {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Numbers
            (Self::Int(lhs), Self::Int(rhs)) => Self::Int(lhs.saturating_add(rhs)),
            (Self::Float(lhs), Self::Float(rhs)) => Self::Float(lhs + rhs),
            (Self::Int(lhs), Self::Float(rhs)) => Self::Float(lhs as f64 + rhs),
            (Self::Float(lhs), Self::Int(rhs)) => Self::Float(lhs + rhs as f64),

            // Strings
            (Self::String(lhs), Self::String(rhs)) => Self::String(lhs + &rhs),

            // Paths
            (Self::Path(lhs), Self::Path(rhs)) => Self::Path(lhs.join(rhs)),
            (Self::String(lhs), Self::Path(rhs)) => Self::Path(PathBuf::from(lhs).join(rhs)),
            (Self::Path(lhs), Self::String(rhs)) => Self::Path(lhs.join(rhs)),

            // Arrays and objects
            (Self::Array(mut lhs), Self::Array(rhs)) => {
                lhs.extend(rhs);
                Self::Array(lhs)
            }
            (Self::Object(mut lhs), Self::Object(rhs)) => {
                for (key, value) in rhs {
                    lhs.insert(key, value);
                }
                Self::Object(lhs)
            }

            _ => Self::Null,
        }
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        let span = merge_spans(self.span, rhs.span);
        Value::new(self.kind + rhs.kind, span)
    }
}

impl Sub for ValueKind {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Numbers
            (Self::Int(lhs), Self::Int(rhs)) => Self::Int(lhs.saturating_sub(rhs)),
            (Self::Float(lhs), Self::Float(rhs)) => Self::Float(lhs - rhs),
            (Self::Int(lhs), Self::Float(rhs)) => Self::Float(lhs as f64 - rhs),
            (Self::Float(lhs), Self::Int(rhs)) => Self::Float(lhs - rhs as f64),

            _ => Self::Null,
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        let span = crate::utils::merge_spans(self.span, rhs.span);
        Value::new(self.kind - rhs.kind, span)
    }
}

impl Mul for ValueKind {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Numbers
            (Self::Int(lhs), Self::Int(rhs)) => Self::Int(lhs.saturating_mul(rhs)),
            (Self::Float(lhs), Self::Float(rhs)) => Self::Float(lhs * rhs),
            (Self::Int(lhs), Self::Float(rhs)) => Self::Float(lhs as f64 * rhs),
            (Self::Float(lhs), Self::Int(rhs)) => Self::Float(lhs * rhs as f64),

            // Repeat strings
            (Self::String(lhs), Self::Int(rhs)) => {
                if let Ok(rhs) = rhs.try_into() {
                    Self::String(lhs.repeat(rhs))
                } else {
                    // Return original string if `rhs` can't be converted to a usize (if `rhs` is negative).
                    Self::String(lhs)
                }
            }

            _ => Self::Null,
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        let span = crate::utils::merge_spans(self.span, rhs.span);
        Value::new(self.kind * rhs.kind, span)
    }
}

impl Div for ValueKind {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Any combination of Int or Float
            (Self::Int(lhs), Self::Int(rhs)) => {
                if rhs == 0 {
                    Self::Null
                } else {
                    Self::Float(lhs as f64 / rhs as f64)
                }
            }
            (Self::Float(lhs), Self::Float(rhs)) => {
                if rhs == 0.0 {
                    Self::Null
                } else {
                    Self::Float(lhs / rhs)
                }
            }
            (Self::Int(lhs), Self::Float(rhs)) => {
                if rhs == 0.0 {
                    Self::Null
                } else {
                    Self::Float(lhs as f64 / rhs)
                }
            }
            (Self::Float(lhs), Self::Int(rhs)) => {
                if rhs == 0 {
                    Self::Null
                } else {
                    Self::Float(lhs / rhs as f64)
                }
            }

            _ => Self::Null,
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        let span = crate::utils::merge_spans(self.span, rhs.span);
        Value::new(self.kind / rhs.kind, span)
    }
}

impl Rem for ValueKind {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Numbers
            (Self::Int(lhs), Self::Int(rhs)) => {
                if rhs == 0 {
                    Self::Null
                } else {
                    Self::Int(lhs % rhs)
                }
            }
            (Self::Float(lhs), Self::Float(rhs)) => {
                if rhs == 0.0 {
                    Self::Null
                } else {
                    Self::Float(lhs % rhs)
                }
            }
            (Self::Int(lhs), Self::Float(rhs)) => {
                if rhs == 0.0 {
                    Self::Null
                } else {
                    Self::Float(lhs as f64 % rhs)
                }
            }
            (Self::Float(lhs), Self::Int(rhs)) => {
                if rhs == 0 {
                    Self::Null
                } else {
                    Self::Float(lhs % rhs as f64)
                }
            }

            _ => Self::Null,
        }
    }
}

impl Rem for Value {
    type Output = Value;

    fn rem(self, rhs: Self) -> Self::Output {
        let span = crate::utils::merge_spans(self.span, rhs.span);
        Value::new(self.kind % rhs.kind, span)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.kind, &other.kind) {
            (ValueKind::Int(lhs), ValueKind::Int(rhs)) => lhs.cmp(rhs),
            (ValueKind::Float(lhs), ValueKind::Float(rhs)) => (*lhs).total_cmp(rhs),
            (ValueKind::Int(lhs), ValueKind::Float(rhs)) => (*lhs as f64).total_cmp(rhs),
            (ValueKind::Float(lhs), ValueKind::Int(rhs)) => lhs.total_cmp(&(*rhs as f64)),
            (ValueKind::String(lhs), ValueKind::String(rhs)) => lhs.cmp(rhs),

            _ => Ordering::Equal,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(test)]
        if self.span != other.span {
            return false;
        }

        match (&self.kind, &other.kind) {
            (ValueKind::Null, ValueKind::Null) => true,
            (ValueKind::Boolean(lhs), ValueKind::Boolean(rhs)) => lhs == rhs,
            (ValueKind::Boolean(bool), other) => *bool && other.is_truthy(),
            (ValueKind::Int(lhs), ValueKind::Int(rhs)) => lhs == rhs,
            (ValueKind::Int(lhs), ValueKind::Float(rhs)) => *lhs == (*rhs as isize),
            (ValueKind::Float(lhs), ValueKind::Float(rhs)) => lhs == rhs,
            (ValueKind::Float(lhs), ValueKind::Int(rhs)) => *lhs == (*rhs as f64),
            (ValueKind::String(lhs), ValueKind::String(rhs)) => lhs == rhs,
            (ValueKind::Array(lhs), ValueKind::Array(rhs)) => lhs == rhs,
            (ValueKind::Object(lhs), ValueKind::Object(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl Eq for Value {}

pub type Error = crate::Error<ErrorKind>;

#[derive(Error, Diagnostic, Debug)]
#[error("Runtime error")]
pub enum ErrorKind {
    #[error("This variable is not in scope")]
    #[diagnostic(code(tl::runtime::expr))]
    VariableNotInScope {
        #[label("Could not find this variable in scope")]
        variable: SourceSpan,
    },

    #[error("Mismatch in number of function arguments")]
    #[diagnostic(code(tl::runtime::call))]
    ArgsMismatch {
        len: usize,

        #[label("Supposed to have {len} argument{s}", s = if *len == 1 { "" } else { "s" })]
        args: SourceSpan,
    },

    #[error("Index out of bounds")]
    #[diagnostic(code(tl::runtime::expr))]
    IndexOutOfBounds {
        length: usize,

        #[label("Length is {length}")]
        index: SourceSpan,
    },

    #[error("Mismatched types, expected {expected}, got {got}")]
    #[diagnostic(code(tl::runtime::expr))]
    MismatchedTypes { expected: String, got: String },

    #[error(transparent)]
    ParseError(#[from] ast::types::Error),

    #[error(transparent)]
    IOError(#[from] io::Error),

    #[cfg(feature = "toml")]
    #[error(transparent)]
    TomlParsingError(#[from] toml::de::Error),
}

impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
