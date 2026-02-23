use crate::{
    parser::ast::types::{Expr, ExprKind},
    runtime::{Error, ErrorKind, Value, ValueKind},
};

impl super::Scope {
    pub(super) fn type_check(&self, value: &Value, ty_expr: Expr) -> Result<(), Error> {
        let piped_call = Expr::new(
            ExprKind::PipedCall {
                input: Expr::boxed_value(value.clone(), value.span),
                base: Box::new(ty_expr.clone()),
                args: vec![],
            },
            value.span,
        );

        let ty_value = self.eval_expr(&ty_expr)?;

        let result = self.eval_expr(&piped_call)?;

        match result.kind {
            ValueKind::Bool(true) => Ok(()),
            ValueKind::Bool(false) => {
                let expected = if let Some(doc) = ty_value.metadata.doc {
                    doc.clone()
                } else {
                    match &ty_expr.kind {
                        ExprKind::Identifier(name) => name.clone(),
                        _ => format!("{ty_expr}"),
                    }
                };

                Err(Error::new(
                    ErrorKind::MismatchedTypes {
                        expected,
                        got: value.type_of().to_owned(),
                    },
                    (*self.0.source).clone(),
                    value.span,
                ))
            }
            other => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "bool".to_owned(),
                    got: other.type_of().to_owned(),
                },
                (*self.0.source).clone(),
                value.span,
            )),
        }
    }
}
