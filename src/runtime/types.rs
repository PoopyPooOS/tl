use crate::parser::ast::types::Statement;
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Function {
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
    NativeFunction {
        parameters: Vec<String>,
        body: fn(Vec<Value>) -> Option<Value>,
    },
}

impl Value {
    #[must_use]
    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Function { .. } => "function",
            Value::NativeFunction { .. } => "native function",
        }
    }

    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n > 0,
            Value::Float(f) => *f > 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(map) => !map.is_empty(),
            Value::Function { .. } | Value::NativeFunction { .. } | Value::Null => false,
        }
    }
}

macro_rules! impl_values {
    ($($type1:ident($type2:ty)),*) => {
        $(
            impl From<$type2> for Value {
                fn from(value: $type2) -> Self {
                    Self::$type1(value)
                }
            }
        )*
    };
}

impl_values!(
    Boolean(bool),
    Number(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>)
);

impl std::ops::Add<Value> for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => (lhs + rhs).into(),
            (Value::Float(lhs), Value::Float(rhs)) => (lhs + rhs).into(),
            (Value::String(lhs), Value::String(rhs)) => (lhs + &rhs).into(),
            _ => todo!(),
        }
    }
}

impl std::ops::Sub<Value> for Value {
    type Output = Value;

    fn sub(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => (lhs - rhs).into(),
            (Value::Float(lhs), Value::Float(rhs)) => (lhs - rhs).into(),
            _ => todo!(),
        }
    }
}

impl std::ops::Mul<Value> for Value {
    type Output = Value;

    fn mul(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => (lhs * rhs).into(),
            (Value::Float(lhs), Value::Float(rhs)) => (lhs * rhs).into(),
            _ => todo!(),
        }
    }
}

impl std::ops::Div<Value> for Value {
    type Output = Value;

    fn div(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => (lhs / rhs).into(),
            (Value::Float(lhs), Value::Float(rhs)) => (lhs / rhs).into(),
            _ => todo!(),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Number(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
            Value::Boolean(v) => write!(f, "{v}"),
            Value::Array(v) => write!(f, "{v:?}"),
            Value::Object(v) => write!(f, "{v:?}"),
            Value::Function { .. } => write!(f, "<function>"),
            Value::NativeFunction { .. } => write!(f, "<native function>"),
        }
    }
}
