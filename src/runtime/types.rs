use crate::parser::ast::types::Statement;
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[cfg(feature = "serde")]
use serde::{
    de::{self, Visitor},
    de::{MapAccess, SeqAccess},
    forward_to_deserialize_any, Deserializer,
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
            Value::Number(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(map) => !map.is_empty(),
            Value::Function { .. } | Value::NativeFunction { .. } | Value::Null => false,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserializer<'de> for Value {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Boolean(b) => visitor.visit_bool(b),
            Value::Number(n) => visitor.visit_i64(n),
            Value::Float(f) => visitor.visit_f64(f),
            Value::String(s) => visitor.visit_string(s),
            Value::Array(arr) => {
                let seq = ValueSeq { iter: arr.into_iter() };
                visitor.visit_seq(seq)
            }
            Value::Object(map) => {
                let map = ValueMap {
                    iter: map.into_iter(),
                    value: None,
                };
                visitor.visit_map(map)
            }
            Value::Function { .. } | Value::NativeFunction { .. } => Err(de::Error::custom("Function types cannot be deserialized")),
        }
    }

    // Forward other methods to deserialize_any
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

// Helper struct for sequences
#[cfg(feature = "serde")]
struct ValueSeq {
    iter: std::vec::IntoIter<Value>,
}

#[cfg(feature = "serde")]
impl<'de> SeqAccess<'de> for ValueSeq {
    type Error = de::value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}

// Helper struct for maps
#[cfg(feature = "serde")]
struct ValueMap {
    iter: std::collections::hash_map::IntoIter<String, Value>,
    value: Option<Value>,
}

#[cfg(feature = "serde")]
impl<'de> MapAccess<'de> for ValueMap {
    type Error = de::value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Value::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(de::Error::custom("Value expected after key")),
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
