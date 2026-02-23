use crate::parser::ast::types::{Expr, ExprKind, Literal};
use colored::{ColoredString, Colorize};
use miette::SourceSpan;
use std::fmt::{self, Display, Write};

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&pretty_print_expr(self, 0)?)
    }
}

type FmtResult = Result<String, fmt::Error>;

pub(crate) fn pretty_print_expr(expr: &Expr, indent: usize) -> FmtResult {
    let pad = "  ".repeat(indent);
    let mut out = String::new();

    macro_rules! fmt {
        ($value:expr, $span:expr $(,)?) => {{
            writeln!(
                out,
                "{pad}{value} {span}",
                value = $value,
                span = pretty_print_span($span),
            )?
        }};
        (tuple $name:expr, $value:expr, $span:expr $(,)?) => {{
            writeln!(
                out,
                "{pad}{name}{}{value}{} {span}",
                "(".dimmed(),
                ")".dimmed(),
                name = $name,
                value = $value,
                span = pretty_print_span($span),
            )?
        }};
        (array $name:expr, $span:expr, $item:ident => $array:expr => $value:expr) => {{
            writeln!(
                out,
                "{pad}{name} {span} {}",
                "[".dimmed(),
                name = $name,
                span = pretty_print_span($span),
            )?;

            for $item in $array {
                writeln!(out, "{pad}  {}", $value)?;
            }

            writeln!(out, "{pad}{}", "]".dimmed())?;
        }};
        (struct $name:expr, $span:expr, { $($key:expr => $value:expr),* $(,)? } ) => {{
            fmt!(struct_start $name, $span);
            $(
                fmt!(struct_field $key, $value);
            )*
            fmt!(struct_end);
        }};
        (struct_start $name:expr, $span:expr $(,)?) => {{
            writeln!(
                out,
                "{pad}{name} {span} {}",
                "{".dimmed(),
                name = $name,
                span = pretty_print_span($span),
            )?;
        }};
        (struct_field $key:expr, $value:expr $(,)?) => {{
            writeln!(
                out,
                "{pad}  {key}{colon} {value}",
                key = $key,
                colon = ":".dimmed(),
                value = $value,
            )?;
        }};
        (struct_end) => {{
            writeln!(out, "{pad}{}", "}".dimmed())?;
        }};
        (newline) => {{
            writeln!(out)?;
        }}
    }

    match &expr.kind {
        ExprKind::Not(inner_expr) => {
            fmt!(tuple "Not".bright_magenta(), &pretty_print_expr(inner_expr, indent.saturating_add(1))?.trim(), expr.span);
        }
        ExprKind::Parenthesized(inner_expr) => {
            fmt!(tuple "Paranthesized".bright_magenta(), &pretty_print_expr(inner_expr, indent.saturating_add(1))?.trim(), expr.span);
        }
        ExprKind::Literal(lit) => match lit {
            Literal::Null => fmt!("null".yellow(), expr.span),
            Literal::Int(v) => fmt!(tuple
                "Int".bright_blue(),
                v.to_string().yellow(),
                expr.span
            ),
            Literal::Float(v) => fmt!(tuple
                "Float".bright_blue(),
                v.to_string().yellow(),
                expr.span
            ),
            Literal::Bool(v) => fmt!(tuple
                "Bool".bright_blue(),
                v.to_string().yellow(),
                expr.span
            ),
            Literal::String(v) => fmt!(tuple
                "String".bright_blue(),
                format!("\"{v}\"").green(),
                expr.span
            ),
            Literal::InterpolatedString(v) => {
                fmt!(array "InterpolatedString".bright_blue(), expr.span,
                    item => v => &pretty_print_expr(item, indent.saturating_add(1))?.trim()
                );
            }
            Literal::Path(v) => fmt!(tuple
                "Path".bright_blue(),
                v.display().to_string().blue(),
                expr.span
            ),
            Literal::InterpolatedPath(v) => {
                fmt!(array "InterpolatedPath".bright_blue(), expr.span,
                    item => v => &pretty_print_expr(item, indent.saturating_add(1))?.trim()
                );
            }
            Literal::Array(v) => {
                fmt!(array "Array".bright_blue(), expr.span,
                    item => v => &pretty_print_expr(item, indent.saturating_add(1))?.trim()
                );
            }
            Literal::Object(v) => {
                fmt!(struct_start "Object".bright_blue(), expr.span);

                for (key, value) in v {
                    writeln!(out, "{pad}  {} {}", "Element".cyan(), "{".dimmed())?;
                    write!(out, "  ")?;
                    fmt!(
                        struct_field
                        "key",
                        pretty_print_expr(key, indent.saturating_add(2))?.trim()
                    );
                    write!(out, "  ")?;
                    fmt!(
                        struct_field
                        "value",
                        pretty_print_expr(value, indent.saturating_add(2))?.trim()
                    );
                    write!(out, "  ")?;
                    fmt!(struct_end);
                }

                fmt!(struct_end);
            }
        },
        ExprKind::Identifier(name) => {
            fmt!(tuple "Identifier".bright_cyan(), name.yellow(), expr.span);
        }
        ExprKind::BinaryOp {
            left,
            operator,
            right,
        } => {
            fmt!(struct "BinaryOp".bright_blue(), expr.span, {
                "left" => pretty_print_expr(left, indent.saturating_add(1))?.trim(),
                "operator" => operator.to_string().red().bold(),
                "right" => pretty_print_expr(right, indent.saturating_add(1))?.trim(),
            });
        }
        ExprKind::ArrayIndex { base, index } => {
            fmt!(struct "ArrayIndex".bright_blue(), expr.span, {
                "base" => pretty_print_expr(base, indent.saturating_add(1))?.trim(),
                "index" => pretty_print_expr(index, indent.saturating_add(1))?.trim(),
            });
        }
        ExprKind::MemberAccess { base, field } => {
            fmt!(struct "MemberAccess".bright_blue(), expr.span, {
                "base" => pretty_print_expr(base, indent.saturating_add(1))?.trim(),
                "field" => field.yellow(),
            });
        }
        ExprKind::Function { args, ret_ty, expr } => {
            fmt!(struct_start "Function".bright_blue(), expr.span);

            for arg in args {
                fmt!(struct_field arg.name.magenta(), pretty_print_expr(&arg.ty, indent.saturating_add(1))?.trim());
            }

            fmt!(struct_field "returns", pretty_print_expr(ret_ty, indent)?.trim());
            fmt!(newline);
            fmt!(struct_field "expr", pretty_print_expr(expr, indent.saturating_add(1))?.trim());

            fmt!(struct_end);
        }
        ExprKind::Call { base, args } => {
            fmt!(struct_start "Call".bright_blue(), expr.span);

            fmt!(struct_field "base", pretty_print_expr(base, indent.saturating_add(1))?.trim());
            fmt!(array "args", expr.span, arg => args => pretty_print_expr(arg, indent.saturating_add(1))?.trim());

            fmt!(struct_end);
        }
        ExprKind::PipedCall { input, base, args } => {
            fmt!(struct_start "PipedCall".bright_blue(), expr.span);

            fmt!(struct_field "input", pretty_print_expr(input, indent.saturating_add(1))?.trim());
            fmt!(struct_field "base", pretty_print_expr(base, indent.saturating_add(1))?.trim());
            fmt!(array "args", expr.span, arg => args => pretty_print_expr(arg, indent.saturating_add(1))?.trim());

            fmt!(struct_end);
        }
        ExprKind::With { object, expr: body } => {
            fmt!(struct_start "With".bright_magenta(), expr.span);

            fmt!(struct_field "object", pretty_print_expr(object, indent.saturating_add(1))?.trim());
            fmt!(newline);
            fmt!(struct_field "expr", pretty_print_expr(body, indent.saturating_add(1))?.trim());

            fmt!(struct_end);
        }
        ExprKind::Value(v) => {
            // This never makes it into an AST, only for internal value-passing
            fmt!("Value", v.span);
        }
    }

    Ok(out)
}

pub(crate) fn pretty_print_span(span: SourceSpan) -> ColoredString {
    format!("{}:{}", span.offset(), span.len()).dimmed()
}
