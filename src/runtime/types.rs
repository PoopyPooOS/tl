#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

use super::ValueResult;
use crate::parser::ast::{self, types::Statement};
use logger::{warn, Log, LogLevel};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display},
    io,
    ops::{Add, Div, Index, Mul, Rem, Sub},
    path::PathBuf,
};

#[derive(Debug, Default, Clone)]
pub enum Value {
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
        body: Vec<Statement>,
    },
}

#[macro_export]
macro_rules! object {
    ($($key:ident = $val:expr),* $(,)?) => {
        Value::Object(std::collections::BTreeMap::from([
            $(
                (stringify!($key).to_owned(), $val),
            )*
        ]))
    };
}

impl Value {
    pub fn type_of(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Boolean(_) => "boolean",
            Self::Int(_) => "number",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::Path(_) => "path",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
            Self::Function { .. } => "function",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Boolean(b) => *b,
            Self::Int(n) => *n > 0,
            Self::Float(f) => *f > 0.0,
            Self::String(s) => !s.is_empty(),
            Self::Path(p) => !p.exists(),
            Self::Array(arr) => !arr.is_empty(),
            Self::Object(map) => !map.is_empty(),
            Self::Function { .. } | Self::Null => false,
        }
    }

    pub fn and(&self, rhs: &Self) -> bool {
        self.is_truthy() && rhs.is_truthy()
    }

    pub fn or(&self, rhs: &Self) -> bool {
        self.is_truthy() || rhs.is_truthy()
    }

    #[must_use]
    pub fn access(&self, rhs: impl Into<String>) -> Self {
        match self {
            Value::Object(v) => v.get(&rhs.into()).unwrap_or(&Value::Null).clone(),
            _ => Value::Null,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::Boolean(v) => f.write_str(&format!("{v}")),
            Value::Int(v) => f.write_str(&format!("{v}")),
            Value::Float(v) => f.write_str(&format!("{v}")),
            Value::String(v) => f.write_str(v),
            Value::Path(v) => f.write_str(&v.display().to_string()),
            Value::Array(v) => {
                let formatted = v.iter().map(ToString::to_string).collect::<Vec<_>>();
                f.write_str(&format!("[ {} ]", formatted.join(" ")))
            }
            Value::Object(v) => {
                let formatted = v
                    .iter()
                    .map(|(k, v)| format!("{k} = {v}"))
                    .collect::<Vec<_>>();
                f.write_str(&format!("{{ {} }}", formatted.join("; ")))
            }
            Value::Function { .. } => f.write_str("function"),
        }
    }
}

impl Index<usize> for Value {
    type Output = Self;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Value::Array(v) => v.get(index).unwrap_or(&Value::Null),
            _ => &Value::Null,
        }
    }
}

impl Add for Value {
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

impl Sub for Value {
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

impl Mul for Value {
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

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Numbers
            (Self::Int(lhs), Self::Int(rhs)) => {
                if rhs == 0 {
                    Self::Null
                } else {
                    Self::Int(lhs.saturating_div(rhs))
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

impl Rem for Value {
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

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Int(lhs), Self::Int(rhs)) => lhs.cmp(rhs),
            (Self::Float(lhs), Self::Float(rhs)) => (*lhs).total_cmp(rhs),
            (Self::Int(lhs), Self::Float(rhs)) => (*lhs as f64).total_cmp(rhs),
            (Self::Float(lhs), Self::Int(rhs)) => lhs.total_cmp(&(*rhs as f64)),
            (Self::String(lhs), Self::String(rhs)) => lhs.cmp(rhs),

            _ => Ordering::Equal,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(lhs), Value::Boolean(rhs)) => lhs == rhs,
            (Value::Boolean(bool), other) => *bool && other.is_truthy(),
            (Value::Int(lhs), Value::Int(rhs)) => lhs == rhs,
            (Value::Int(lhs), Value::Float(rhs)) => *lhs == (*rhs as isize),
            (Value::Float(lhs), Value::Float(rhs)) => lhs == rhs,
            (Value::Float(lhs), Value::Int(rhs)) => *lhs == (*rhs as f64),
            (Value::String(lhs), Value::String(rhs)) => lhs == rhs,
            (Value::Array(lhs), Value::Array(rhs)) => lhs == rhs,
            (Value::Object(lhs), Value::Object(rhs)) => lhs == rhs,
            (Value::Function { .. }, _) | (_, Value::Function { .. }) => {
                warn!("Functions can not be compared");
                false
            }
            _ => false,
        }
    }
}

impl Eq for Value {}

pub type NativeFn = Box<dyn Fn(Vec<Value>) -> ValueResult>;
pub enum NativeFunction {
    /// Has strict parameter amount requirements.
    Strict { params: usize, func: NativeFn },
    /// Doesn't have strict parameter amount requirements.
    Loose(NativeFn),
}

pub type Error = crate::Error<ErrorType>;

#[derive(Debug)]
pub enum ErrorType {
    VariableDoesntExist(String),
    FieldDoesntExist(String),
    FunctionDoesntExist(String),

    CanNotAccessWithNonIdent,

    /// (`index`, `length`)
    IndexOutOfBounds(usize, usize),

    /// (`function_name`, `params_len`, `args_len`)
    ArgsMismatch(String, usize, usize),

    NativeFnError(String),

    ParseError(ast::types::Error),

    #[cfg(feature = "toml")]
    TomlParsingError(toml::de::Error),

    IOError(io::Error),
}

impl From<Error> for Log {
    fn from(value: Error) -> Self {
        let log = Log {
            level: LogLevel::Error,
            message: "Unknown runtime error".into(),
            location: value.location,
            hint: None,
        };

        log.message(match value.error_type {
            ErrorType::VariableDoesntExist(name) => {
                format!("The variable '{name}' does not exist.")
            }
            ErrorType::FieldDoesntExist(name) => {
                format!("The field '{name}' does not exist.")
            }
            ErrorType::FunctionDoesntExist(name) => {
                format!("The function '{name}' does not exist.")
            }
            ErrorType::CanNotAccessWithNonIdent => {
                "Can not access fields with expressions other than identifiers".into()
            }
            ErrorType::IndexOutOfBounds(index, length) => {
                format!("Index out of bounds: index is {index} but the length is {length}")
            }
            ErrorType::ArgsMismatch(name, params_len, args_len) => {
                format!("Function '{name}' has {params_len} parameter{}, but {args_len} argument{} {} provided", if params_len == 1 { "" } else { "s" }, if args_len == 1 { "" } else { "s" }, if args_len == 1 { "was" } else { "were" })
            },
            ErrorType::NativeFnError(v) => v,
            ErrorType::ParseError(err) => {
                return Self::from(err);
            }
            #[cfg(feature = "toml")]
            ErrorType::TomlParsingError(err) => format!("{err}"),
            ErrorType::IOError(error) => format!("IO error: {error}"),
        })
    }
}

impl PartialEq for ErrorType {
    fn eq(&self, other: &Self) -> bool {
        use ErrorType as E;

        match (self, other) {
            (E::VariableDoesntExist(r_ident), E::VariableDoesntExist(l_ident))
                if r_ident == l_ident =>
            {
                true
            }
            (E::FieldDoesntExist(r_ident), E::FieldDoesntExist(l_ident)) if r_ident == l_ident => {
                true
            }
            (E::FunctionDoesntExist(r_ident), E::FunctionDoesntExist(l_ident))
                if r_ident == l_ident =>
            {
                true
            }
            (E::CanNotAccessWithNonIdent, E::CanNotAccessWithNonIdent) => true,
            (E::IndexOutOfBounds(r_index, r_len), E::IndexOutOfBounds(l_index, l_len))
                if r_index == l_index && r_len == l_len =>
            {
                true
            }
            (
                E::ArgsMismatch(r_name, r_params, r_args),
                E::ArgsMismatch(l_name, l_params, l_args),
            ) if r_name == l_name && r_params == l_params && r_args == l_args => true,

            _ => false,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::new(ErrorType::IOError(value), None)
    }
}

impl From<ast::types::Error> for Error {
    fn from(value: ast::types::Error) -> Self {
        let location = value.location.clone();
        Self::new(ErrorType::ParseError(value), location)
    }
}

#[cfg(feature = "toml")]
impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::new(ErrorType::TomlParsingError(value), None)
    }
}
