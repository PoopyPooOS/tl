use crate::{
    parser::ast::types::{BinaryOperator, Literal},
    runtime::Value,
};
use derive_more::Display;
use miette::SourceSpan;

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SourceSpan,
}

impl Default for Expr {
    fn default() -> Self {
        Self::lit(Literal::Null, SourceSpan::new(0.into(), 0))
    }
}

impl Expr {
    pub const fn new(kind: ExprKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub const fn lit(literal: Literal, span: SourceSpan) -> Self {
        Self::new(ExprKind::Literal(literal), span)
    }

    pub fn ident(ident: impl AsRef<str>, span: SourceSpan) -> Self {
        Self::new(ExprKind::Identifier(ident.as_ref().to_owned()), span)
    }

    pub fn boxed(kind: ExprKind, span: SourceSpan) -> Box<Self> {
        Box::new(Self::new(kind, span))
    }

    pub fn boxed_lit(literal: Literal, span: SourceSpan) -> Box<Self> {
        Box::new(Self::lit(literal, span))
    }

    pub fn boxed_ident(ident: impl AsRef<str>, span: SourceSpan) -> Box<Self> {
        Box::new(Self::ident(ident, span))
    }

    pub fn value(value: Value, span: SourceSpan) -> Self {
        Self::new(ExprKind::Value(Box::new(value)), span)
    }

    pub fn boxed_value(value: Value, span: SourceSpan) -> Box<Self> {
        Box::new(Self::new(ExprKind::Value(Box::new(value)), span))
    }

    pub fn as_ident(&self) -> Option<String> {
        match &self.kind {
            ExprKind::Identifier(ident) => Some(ident.clone()),
            ExprKind::MemberAccess { field, .. } => Some(field.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    Not(Box<Expr>),
    Parenthesized(Box<Expr>),
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
    ArrayIndex {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    MemberAccess {
        base: Box<Expr>,
        field: String,
    },
    Function {
        args: Vec<FnArg>,
        ret_ty: Box<Expr>,
        expr: Box<Expr>,
    },
    Call {
        base: Box<Expr>,
        args: Vec<Expr>,
    },
    PipedCall {
        input: Box<Expr>,
        base: Box<Expr>,
        args: Vec<Expr>,
    },
    With {
        object: Box<Expr>,
        expr: Box<Expr>,
    },

    Value(Box<Value>),
}

#[derive(Debug, Clone, PartialEq, Display)]
#[display("{name}: {ty}")]
pub struct FnArg {
    pub name: String,
    pub ty: Expr,
}

impl FnArg {
    pub fn new(name: impl Into<String>, ty: Expr) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}
