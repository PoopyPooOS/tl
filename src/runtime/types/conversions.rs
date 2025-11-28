use crate::runtime::{Value, ValueKind};
use indexmap::IndexMap;
use std::path::PathBuf;

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

impl<T: Into<Value>> From<IndexMap<String, T>> for ValueKind {
    fn from(val: IndexMap<String, T>) -> Self {
        Self::Object(val.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl From<super::builtin::Builtin> for ValueKind {
    fn from(val: super::builtin::Builtin) -> Self {
        Self::Builtin(val)
    }
}

impl From<()> for Value {
    fn from(val: ()) -> Self {
        Self::new_builtin(ValueKind::from(val))
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Self::new_builtin(ValueKind::from(val))
    }
}

impl From<isize> for Value {
    fn from(val: isize) -> Self {
        Self::new_builtin(ValueKind::from(val))
    }
}

impl From<f64> for Value {
    fn from(val: f64) -> Self {
        Self::new_builtin(ValueKind::from(val))
    }
}

impl<'a> From<&'a str> for Value {
    fn from(val: &'a str) -> Self {
        Self::new_builtin(ValueKind::from(val))
    }
}

impl From<String> for Value {
    fn from(val: String) -> Self {
        Self::new_builtin(ValueKind::from(val))
    }
}

impl From<PathBuf> for Value {
    fn from(val: PathBuf) -> Self {
        Self::new_builtin(ValueKind::from(val))
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(val: Vec<T>) -> Self {
        Self::new_builtin(ValueKind::from(val))
    }
}

impl<T: Into<Value>> From<IndexMap<String, T>> for Value {
    fn from(val: IndexMap<String, T>) -> Self {
        Self::new_builtin(ValueKind::from(val))
    }
}

impl From<super::builtin::Builtin> for Value {
    fn from(val: super::builtin::Builtin) -> Self {
        Self::new_builtin(ValueKind::from(val))
    }
}

#[macro_export]
macro_rules! object {
    ($($key:ident: $val:expr),* $(,)?) => {
        Value::new_builtin($crate::runtime::types::ValueKind::Object($crate::indexmap::IndexMap::from([
            $(
                (stringify!($key).to_owned(), Value::new_builtin($val.into())),
            )*
        ])))
    };
}
