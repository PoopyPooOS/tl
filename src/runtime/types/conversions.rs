use crate::runtime::{Builtin, Value, ValueKind};
use indexmap::IndexMap;
use std::path::PathBuf;

macro_rules! impl_conversion {
    ($t:ty, $var:ident => $kind_expr:expr) => {
        impl From<$t> for ValueKind {
            #[allow(unused_variables)]
            fn from($var: $t) -> Self {
                $kind_expr
            }
        }
        impl From<$t> for Value {
            fn from($var: $t) -> Self {
                Self::new_builtin(ValueKind::from($var))
            }
        }
    };
    ($t:ty, $var:ident => $kind_expr:expr, $($gen:tt)*) => {
        impl<$($gen)*> From<$t> for ValueKind {
            fn from($var: $t) -> Self {
                $kind_expr
            }
        }
        impl<$($gen)*> From<$t> for Value {
            fn from($var: $t) -> Self {
                Self::new_builtin(ValueKind::from($var))
            }
        }
    };
}

impl_conversion!((), value => Self::Null);
impl_conversion!(bool, v => Self::Bool(v));
impl_conversion!(isize, v => Self::Int(v));
impl_conversion!(f64, v => Self::Float(v));
impl_conversion!(String, v => Self::String(v));
impl_conversion!(PathBuf, v => Self::Path(v));
impl_conversion!(Builtin, v => Self::Builtin(v));
impl_conversion!(&'a str, v => Self::String(v.into()), 'a);
impl_conversion!(Vec<T>, v => Self::Array(v.into_iter().map(Into::into).collect()), T: Into<Value>);
impl_conversion!(IndexMap<String, T>, v => Self::Object(v.into_iter().map(|(k, v)| (k, v.into())).collect()), T: Into<Value>);

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
