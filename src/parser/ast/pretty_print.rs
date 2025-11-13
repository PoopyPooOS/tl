use crate::parser::ast::types::{Expr, ExprKind, Literal};
use colored::Colorize;
use miette::SourceSpan;
use std::fmt::Write;

impl super::Parser {
    pub fn pretty_print_ast(&self, expr: &Expr) -> String {
        self.pretty_print_expr(expr, 0)
    }

    #[allow(clippy::write_with_newline, reason = "This is far easier to read")]
    fn pretty_print_expr(&self, expr: &Expr, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        let mut out = String::new();

        match &expr.kind {
            ExprKind::Not(inner_expr) => {
                let _ = writeln!(
                    out,
                    "{pad}{} {}",
                    "Not".bright_magenta(),
                    self.pretty_print_span(expr.span).dimmed(),
                );
                out.push_str(&self.pretty_print_expr(inner_expr, indent.saturating_add(1)));
            }
            ExprKind::Literal(lit) => {
                let value = match lit {
                    Literal::Null => format!(
                        "{}{} {}",
                        pad,
                        "null".yellow(),
                        self.pretty_print_span(expr.span).dimmed(),
                    ),
                    Literal::Int(v) => format!(
                        "{}{}{}{}{} {}",
                        pad,
                        "Int".bright_blue(),
                        "(".dimmed(),
                        v.to_string().yellow(),
                        ")".dimmed(),
                        self.pretty_print_span(expr.span).dimmed(),
                    ),
                    Literal::Float(v) => format!(
                        "{}{}{}{}{} {}",
                        pad,
                        "Float".bright_blue(),
                        "(".dimmed(),
                        v.to_string().yellow(),
                        ")".dimmed(),
                        self.pretty_print_span(expr.span).dimmed(),
                    ),
                    Literal::Bool(v) => format!(
                        "{}{}{}{}{} {}",
                        pad,
                        "Bool".bright_blue(),
                        "(".dimmed(),
                        v.to_string().yellow(),
                        ")".dimmed(),
                        self.pretty_print_span(expr.span).dimmed(),
                    ),
                    Literal::String(v) => format!(
                        "{}{}{}{}{} {}",
                        pad,
                        "String".bright_blue(),
                        "(".dimmed(),
                        format!("\"{v}\"").green(),
                        ")".dimmed(),
                        self.pretty_print_span(expr.span).dimmed(),
                    ),
                    Literal::InterpolatedString(v) => {
                        let mut s = format!(
                            "{}{} {}\n",
                            pad,
                            "InterpolatedString".bright_blue(),
                            self.pretty_print_span(expr.span).dimmed(),
                        );
                        for item in v {
                            s.push_str(&self.pretty_print_expr(item, indent.saturating_add(1)));
                        }
                        s
                    }
                    Literal::Path(v) => format!(
                        "{}{}{}{}{} {}",
                        pad,
                        "Path".bright_blue(),
                        "(".dimmed(),
                        v.display().to_string().blue(),
                        ")".dimmed(),
                        self.pretty_print_span(expr.span).dimmed(),
                    ),
                    Literal::InterpolatedPath(v) => {
                        let mut s = format!(
                            "{}{} {}\n",
                            pad,
                            "InterpolatedPath".bright_blue(),
                            self.pretty_print_span(expr.span).dimmed(),
                        );
                        for item in v {
                            s.push_str(&self.pretty_print_expr(item, indent.saturating_add(1)));
                        }
                        s
                    }
                    Literal::Array(v) => {
                        let mut s = format!(
                            "{}{} {} {}\n",
                            pad,
                            "Array".bright_blue(),
                            self.pretty_print_span(expr.span).dimmed(),
                            "[".dimmed()
                        );
                        for item in v {
                            s.push_str(&self.pretty_print_expr(item, indent.saturating_add(1)));
                        }
                        let _ = write!(s, "{pad}{}", "]".dimmed());
                        s
                    }
                    Literal::Object(v) => {
                        let mut s = format!(
                            "{}{} {} {}\n",
                            pad,
                            "Object".bright_blue(),
                            self.pretty_print_span(expr.span).dimmed(),
                            "{".dimmed()
                        );
                        for (key, value) in v {
                            let _ = write!(s, "{pad}  {key} {} ", "=".cyan());
                            s.push_str(
                                self.pretty_print_expr(value, indent.saturating_add(1))
                                    .trim_start(),
                            );
                        }
                        let _ = write!(s, "{pad}{}", "}".dimmed());
                        s
                    }
                };

                let _ = writeln!(out, "{value}");
            }
            ExprKind::Identifier(name) => {
                let _ = write!(
                    out,
                    "{pad}{} {} {}\n",
                    "Identifier".bright_cyan(),
                    format!("\"{name}\"").yellow(),
                    self.pretty_print_span(expr.span).dimmed(),
                );
            }
            ExprKind::BinaryOp {
                left,
                operator,
                right,
            } => {
                let _ = writeln!(
                    out,
                    "{pad}{} {}",
                    "BinaryOp".bright_blue(),
                    self.pretty_print_span(expr.span).dimmed(),
                );
                let _ = write!(out, "{pad}  left: ");
                out.push_str(
                    self.pretty_print_expr(left, indent.saturating_add(1))
                        .trim(),
                );
                out.push('\n');
                let _ = writeln!(
                    out,
                    "{pad}  operator: {}",
                    operator.to_string().red().bold()
                );
                let _ = write!(out, "{pad}  right: ");
                out.push_str(
                    self.pretty_print_expr(right, indent.saturating_add(1))
                        .trim(),
                );
                out.push('\n');
            }
            ExprKind::ArrayIndex { base, index } => {
                let _ = writeln!(
                    out,
                    "{pad}{} {}",
                    "ArrayIndex".bright_blue(),
                    self.pretty_print_span(expr.span).dimmed(),
                );
                let _ = write!(out, "{pad}  base: ");
                out.push_str(
                    self.pretty_print_expr(base, indent.saturating_add(1))
                        .trim(),
                );
                out.push('\n');
                let _ = writeln!(out, "{pad}  index: {}", index.to_string().yellow());
            }
            ExprKind::ObjectAccess { base, field } => {
                let _ = writeln!(
                    out,
                    "{pad}{} {}",
                    "ObjectAccess".bright_blue(),
                    self.pretty_print_span(expr.span).dimmed(),
                );
                let _ = write!(out, "{pad}  base: ");
                out.push_str(
                    self.pretty_print_expr(base, indent.saturating_add(1))
                        .trim(),
                );
                out.push('\n');
                let _ = writeln!(out, "{pad}  field: {}", field.yellow());
            }
            ExprKind::FnDecl { args, expr } => {
                let _ = write!(
                    out,
                    "{pad}{} {} {}\n",
                    "FnDecl".bright_blue(),
                    self.pretty_print_span(expr.span).dimmed(),
                    "{".dimmed(),
                );

                for arg in args {
                    let _ = writeln!(out, "{pad}  arg: {}", arg.magenta());
                }

                let _ = write!(out, "{pad}  expr: ");
                out.push_str(
                    self.pretty_print_expr(expr, indent.saturating_add(1))
                        .trim(),
                );
                out.push('\n');

                let _ = writeln!(out, "{pad}{}", "}".dimmed());
            }
            ExprKind::Call { base, args } => {
                let _ = writeln!(
                    out,
                    "{pad}{} {}",
                    "Call".bright_blue(),
                    self.pretty_print_span(expr.span).dimmed(),
                );
                let _ = write!(out, "{pad}  base: ");
                out.push_str(
                    self.pretty_print_expr(base, indent.saturating_add(1))
                        .trim_start(),
                );
                for arg in args {
                    let _ = write!(out, "{pad}  arg: ");
                    out.push_str(
                        self.pretty_print_expr(arg, indent.saturating_add(1))
                            .trim_start(),
                    );
                }
            }
            ExprKind::LetIn {
                bindings,
                expr: body,
            } => {
                let _ = writeln!(
                    out,
                    "{pad}{} {}",
                    "LetIn".bright_magenta(),
                    self.pretty_print_span(expr.span).dimmed(),
                );
                for (name, val) in bindings {
                    let _ = write!(out, "{pad}  {name} {} ", "=".cyan());
                    out.push_str(self.pretty_print_expr(val, indent.saturating_add(1)).trim());
                    out.push('\n');
                }
                let _ = writeln!(out, "\n{pad}  expr:");
                out.push_str(&self.pretty_print_expr(body, indent.saturating_add(2)));
            }
        }

        out
    }

    fn pretty_print_span(&self, span: SourceSpan) -> String {
        let mut line: usize = 1;
        let mut col: usize = 1;
        let mut byte_index = 0;

        for c in self.source.inner().chars() {
            if byte_index == span.offset().saturating_add(span.len()) {
                break;
            }

            if c == '\n' {
                line = line.saturating_add(1);
                col = 1;
            } else {
                col = col.saturating_add(1);
            }

            byte_index = byte_index.saturating_add(c.len_utf8());
        }

        format!("{line}:{col}")
    }
}
