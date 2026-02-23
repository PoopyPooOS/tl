#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

use crate::{
    merge_spans,
    runtime::{Value, ValueKind},
};
use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Rem, Sub},
    path::PathBuf,
};

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
        let span = crate::merge_spans(self.span, rhs.span);
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
        let span = crate::merge_spans(self.span, rhs.span);
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
        let span = crate::merge_spans(self.span, rhs.span);
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
        let span = crate::merge_spans(self.span, rhs.span);
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
        if crate::tests::runtime::IS_ASSERT.with(|v| v.get().is_some()) && self.span != other.span {
            return false;
        }

        match (&self.kind, &other.kind) {
            (ValueKind::Null, ValueKind::Null) => true,
            (ValueKind::Bool(lhs), ValueKind::Bool(rhs)) => lhs == rhs,
            (ValueKind::Bool(bool), other) => *bool && other.is_truthy(),
            (ValueKind::Int(lhs), ValueKind::Int(rhs)) => lhs == rhs,
            (ValueKind::Int(lhs), ValueKind::Float(rhs)) => *lhs == (*rhs as isize),
            (ValueKind::Float(lhs), ValueKind::Float(rhs)) => lhs == rhs,
            (ValueKind::Float(lhs), ValueKind::Int(rhs)) => *lhs == (*rhs as f64),
            (ValueKind::String(lhs), ValueKind::String(rhs)) => lhs == rhs,
            (ValueKind::Array(lhs), ValueKind::Array(rhs)) => lhs == rhs,
            (ValueKind::Object(lhs), ValueKind::Object(rhs)) => lhs == rhs,
            (ValueKind::Thunk { expr: lhs, .. }, ValueKind::Thunk { expr: rhs, .. }) => lhs == rhs,
            _ => false,
        }
    }
}

impl Eq for Value {}
