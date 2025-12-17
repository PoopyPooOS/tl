use crate::parser::ast::types::Expr;
use derive_more::Display;

#[derive(Debug, Clone, PartialEq, Default, Display)]
#[display(rename_all = "lowercase")]
pub enum Type {
    #[default]
    Any,
    Nothing,
    #[display("bool")]
    Boolean,

    Int,
    UInt,
    Float,
    /// Equivalent to `either<int, uint, float>`
    Number,

    String,
    Path,

    #[display("list<{_0}>")]
    List(Box<Type>),
    #[display("object<{}>", _0.iter().map(|(name, ty)| format!("{name}: {ty}")).collect::<Vec<String>>().join(", "))]
    Object(Box<[(String, Type)]>),
    #[display("either<{}>", _0.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "))]
    Either(Box<[Type]>),

    Function,
    #[display("thunk<{_0}>")]
    Thunk(Box<Type>),

    /// User-defined types
    #[display("{}", _0.to_string().trim())]
    Runtime(Expr),
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
