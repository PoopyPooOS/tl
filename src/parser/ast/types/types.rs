use crate::parser::ast::{pretty_print::pretty_print_type, types::Expr};
use derive_more::Display;
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Type {
    #[default]
    Any,
    Nothing,
    Boolean,

    Int,
    UInt,
    Float,
    /// Equivalent to `either<int, uint, float>`
    Number,

    String,
    Path,

    List(Box<Type>),
    Object(Box<[(String, Type)]>),
    Either(Box<[Type]>),

    Function,
    Thunk(Box<Type>),

    /// User-defined types
    Runtime(Expr),
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&pretty_print_type(self, 0)?)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Display)]
#[display("{name}: {ty}")]
pub struct FnArg {
    pub name: String,
    pub ty: Type,
}

impl FnArg {
    pub fn new(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}
