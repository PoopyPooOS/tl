use crate::parser::ast::types::{Expr, ExprKind, Literal, Type};
use colored::{ColoredString, Colorize};
use miette::SourceSpan;
use std::fmt::{self, Display, Write};

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&pretty_print_expr(self, 0)?)
    }
}

pub(crate) fn pretty_print_expr(expr: &Expr, indent: usize) -> Result<String, fmt::Error> {
    let pad = "  ".repeat(indent);
    let mut out = String::new();

    match &expr.kind {
        ExprKind::Not(inner_expr) => {
            writeln!(
                out,
                "{pad}{} {}",
                "Not".bright_magenta(),
                pretty_print_span(expr.span),
            )?;
            out.push_str(&pretty_print_expr(inner_expr, indent.saturating_add(1))?);
        }
        ExprKind::Parenthesized(inner_expr) => {
            writeln!(
                out,
                "{pad}{} {}",
                "Parenthesized".bright_magenta(),
                pretty_print_span(expr.span),
            )?;
            out.push_str(&pretty_print_expr(inner_expr, indent.saturating_add(1))?);
        }
        ExprKind::Literal(lit) => match lit {
            Literal::Null => writeln!(
                out,
                "{}{} {}",
                pad,
                "null".yellow(),
                pretty_print_span(expr.span),
            )?,
            Literal::Int(v) => writeln!(
                out,
                "{}{}{}{}{} {}",
                pad,
                "Int".bright_blue(),
                "(".dimmed(),
                v.to_string().yellow(),
                ")".dimmed(),
                pretty_print_span(expr.span),
            )?,
            Literal::Float(v) => writeln!(
                out,
                "{}{}{}{}{} {}",
                pad,
                "Float".bright_blue(),
                "(".dimmed(),
                v.to_string().yellow(),
                ")".dimmed(),
                pretty_print_span(expr.span),
            )?,
            Literal::Bool(v) => writeln!(
                out,
                "{}{}{}{}{} {}",
                pad,
                "Bool".bright_blue(),
                "(".dimmed(),
                v.to_string().yellow(),
                ")".dimmed(),
                pretty_print_span(expr.span),
            )?,
            Literal::String(v) => writeln!(
                out,
                "{}{}{}{}{} {}",
                pad,
                "String".bright_blue(),
                "(".dimmed(),
                format!("\"{v}\"").green(),
                ")".dimmed(),
                pretty_print_span(expr.span),
            )?,
            Literal::InterpolatedString(v) => {
                writeln!(
                    out,
                    "{}{} {} {}",
                    pad,
                    "InterpolatedString".bright_blue(),
                    pretty_print_span(expr.span),
                    "[".dimmed()
                )?;

                for item in v {
                    writeln!(
                        out,
                        "{pad}  {}",
                        &pretty_print_expr(item, indent.saturating_add(1))?.trim()
                    )?;
                }

                writeln!(out, "{pad}{}", "]".dimmed())?;
            }
            Literal::Path(v) => writeln!(
                out,
                "{}{}{}{}{} {}",
                pad,
                "Path".bright_blue(),
                "(".dimmed(),
                v.display().to_string().blue(),
                ")".dimmed(),
                pretty_print_span(expr.span),
            )?,
            Literal::InterpolatedPath(v) => {
                writeln!(
                    out,
                    "{}{} {}",
                    pad,
                    "InterpolatedPath".bright_blue(),
                    pretty_print_span(expr.span),
                )?;

                for item in v {
                    writeln!(
                        out,
                        "{}",
                        &pretty_print_expr(item, indent.saturating_add(1))?.trim()
                    )?;
                }
            }
            Literal::Array(v) => {
                writeln!(
                    out,
                    "{}{} {} {}",
                    pad,
                    "Array".bright_blue(),
                    pretty_print_span(expr.span),
                    "[".dimmed()
                )?;

                for item in v {
                    writeln!(
                        out,
                        "{}",
                        &pretty_print_expr(item, indent.saturating_add(1))?.trim()
                    )?;
                }

                writeln!(out, "{pad}{}", "]".dimmed())?;
            }
            Literal::Object(v) => {
                writeln!(
                    out,
                    "{}{} {} {}",
                    pad,
                    "Object".bright_blue(),
                    pretty_print_span(expr.span),
                    "{".dimmed()
                )?;

                for (key, value) in v {
                    writeln!(out, "{pad}  {} {}", "Element".cyan(), "{".dimmed())?;
                    writeln!(
                        out,
                        "{pad}    key: {}",
                        pretty_print_expr(key, indent.saturating_add(2))?.trim()
                    )?;
                    writeln!(
                        out,
                        "{pad}    value: {}",
                        pretty_print_expr(value, indent.saturating_add(2))?.trim()
                    )?;
                    writeln!(out, "{pad}  {}", "}".dimmed())?;
                }

                writeln!(out, "{pad}{}", "}".dimmed())?;
            }
        },
        ExprKind::Identifier(name) => {
            writeln!(
                out,
                "{pad}{} {} {}",
                "Identifier".bright_cyan(),
                name.yellow(),
                pretty_print_span(expr.span),
            )?;
        }
        ExprKind::BinaryOp {
            left,
            operator,
            right,
        } => {
            writeln!(
                out,
                "{pad}{} {} {}",
                "BinaryOp".bright_blue(),
                pretty_print_span(expr.span),
                "{".dimmed(),
            )?;

            writeln!(
                out,
                "{pad}  left: {}",
                pretty_print_expr(left, indent.saturating_add(1))?.trim()
            )?;
            writeln!(
                out,
                "{pad}  operator: {}",
                operator.to_string().red().bold()
            )?;
            writeln!(
                out,
                "{pad}  right: {}",
                pretty_print_expr(right, indent.saturating_add(1))?.trim()
            )?;

            writeln!(out, "{pad}{}", "}".dimmed())?;
        }
        ExprKind::ArrayIndex { base, index } => {
            writeln!(
                out,
                "{pad}{} {}",
                "ArrayIndex".bright_blue(),
                pretty_print_span(expr.span),
            )?;

            writeln!(
                out,
                "{pad}  base: {}",
                pretty_print_expr(base, indent.saturating_add(1))?.trim()
            )?;
            writeln!(
                out,
                "{pad}  index: {}",
                pretty_print_expr(index, indent.saturating_add(1))?.trim()
            )?;
        }
        ExprKind::MemberAccess { base, field } => {
            writeln!(
                out,
                "{pad}{} {}",
                "MemberAccess".bright_blue(),
                pretty_print_span(expr.span),
            )?;

            writeln!(
                out,
                "{pad}  base: {}",
                pretty_print_expr(base, indent.saturating_add(1))?.trim()
            )?;
            writeln!(out, "{pad}  field: {}", field.yellow())?;
        }
        ExprKind::Function { args, ret_ty, expr } => {
            writeln!(
                out,
                "{pad}{} {} {}",
                "Function".bright_blue(),
                pretty_print_span(expr.span),
                "{".dimmed(),
            )?;

            for arg in args {
                writeln!(
                    out,
                    "{pad}  arg: {}: {}",
                    arg.name.magenta(),
                    pretty_print_type(&arg.ty, indent.saturating_add(1))?
                )?;
            }

            writeln!(
                out,
                "{pad}  returns: {}\n",
                pretty_print_type(ret_ty, indent)?
            )?;
            writeln!(
                out,
                "{pad}  expr: {}",
                pretty_print_expr(expr, indent.saturating_add(1))?.trim()
            )?;

            writeln!(out, "{pad}{}", "}".dimmed())?;
        }
        ExprKind::Call { base, args } => {
            writeln!(
                out,
                "{pad}{} {}",
                "Call".bright_blue(),
                pretty_print_span(expr.span),
            )?;

            writeln!(
                out,
                "{pad}  base: {}",
                pretty_print_expr(base, indent.saturating_add(1))?.trim()
            )?;
            for arg in args {
                writeln!(
                    out,
                    "{pad}  arg: {}",
                    pretty_print_expr(arg, indent.saturating_add(1))?.trim()
                )?;
            }
        }
        ExprKind::With { object, expr: body } => {
            writeln!(
                out,
                "{pad}{} {}",
                "With".bright_magenta(),
                pretty_print_span(expr.span),
            )?;
            writeln!(
                out,
                "{pad}  object: {}\n",
                pretty_print_expr(object, indent.saturating_add(1))?.trim()
            )?;
            writeln!(
                out,
                "{pad}  expr: {}",
                pretty_print_expr(body, indent.saturating_add(1))?.trim()
            )?;
        }
    }

    Ok(out)
}

pub(crate) fn pretty_print_type(ty: &Type, indent: usize) -> Result<String, fmt::Error> {
    Ok(match ty {
        Type::Any => "any".yellow().to_string(),
        Type::Nothing => "nothing".yellow().to_string(),
        Type::Boolean => "bool".yellow().to_string(),
        Type::Int => "int".yellow().to_string(),
        Type::UInt => "uint".yellow().to_string(),
        Type::Float => "float".yellow().to_string(),
        Type::Number => "number".yellow().to_string(),
        Type::String => "string".yellow().to_string(),
        Type::Path => "path".yellow().to_string(),
        Type::List(ty) => format!(
            "{}{}{}{}",
            "list".yellow(),
            "<".dimmed(),
            pretty_print_type(ty, indent.saturating_add(1))?.trim(),
            ">".dimmed()
        ),
        Type::Object(items) => format!(
            "{}{}{}{}",
            "object".yellow(),
            "<".dimmed(),
            items
                .iter()
                .map(|(name, ty)| -> Result<String, fmt::Error> {
                    Ok(format!(
                        "{name}: {}",
                        pretty_print_type(ty, indent.saturating_add(1))?.trim()
                    ))
                })
                .try_collect::<Vec<String>>()?
                .join(", "),
            ">".dimmed()
        ),
        Type::Either(items) => format!(
            "{}{}{}{}",
            "either".yellow(),
            "<".dimmed(),
            items
                .iter()
                .map(|ty| -> Result<String, fmt::Error> {
                    Ok(pretty_print_type(ty, indent.saturating_add(1))?
                        .trim()
                        .to_owned())
                })
                .try_collect::<Vec<String>>()?
                .join(", "),
            ">".dimmed()
        ),
        Type::Function => "function".yellow().to_string(),
        Type::Thunk(ty) => format!(
            "{}{}{}{}",
            "thunk".yellow(),
            "<".dimmed(),
            pretty_print_type(ty, indent.saturating_add(1))?.trim(),
            ">".dimmed()
        ),
        Type::Runtime(expr) => format!(
            "{}{}{}{}",
            "runtime".yellow(),
            "<".dimmed(),
            pretty_print_expr(expr, indent.saturating_add(1))?.trim(),
            ">".dimmed()
        ),
    })
}

pub(crate) fn pretty_print_span(span: SourceSpan) -> ColoredString {
    format!("{}:{}", span.offset(), span.len()).dimmed()
}
