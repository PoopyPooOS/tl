#![feature(let_chains, macro_metavar_expr, concat_idents)]
#![allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]

pub mod helpers;
pub mod level;
pub mod location;
pub mod utils; // Sharing is caring

pub use colored::{Color, Colorize};
pub use level::LogLevel;
pub use location::Location;
use std::{
    env,
    fmt::{self, Debug, Display, Formatter},
};

pub struct Log {
    pub level: LogLevel,
    pub message: String,
    pub location: Option<Location>,
    pub hint: Option<String>,
}

impl Log {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            location: None,
            hint: None,
        }
    }

    pub fn location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn output(self) {
        print!("{self}");
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let last_line_string = if let Some(ref location) = self.location {
            location.lines.end().to_string()
        } else {
            String::new()
        };

        let padding_size = last_line_string.len() + 1;
        let padding = " ".repeat(padding_size);

        let app_name = env::var("LOGGER_APP_NAME").unwrap_or(String::new());

        // Log level and message
        writeln!(
            f,
            "{}{}",
            if app_name.is_empty() {
                self.level.to_string().color(self.level).bold()
            } else {
                format!("{}[{}]", self.level.to_string().color(self.level), app_name).bold()
            },
            format!(": {}", self.message).bold()
        )?;

        // Location
        if let Some(location) = &self.location {
            writeln!(f, "{}{} {}", &padding[1..], "-->".blue().bold(), location)?;

            // Source
            writeln!(f, "{}{}", padding, "|".blue().bold())?;

            if let Ok(source) = location.read() {
                let source = utils::remove_excess_tabs(source);
                highlight_source(f, source, location, &padding, self.level)?;
            }
        }

        // Hint
        if let Some(hint) = &self.hint {
            if self.location.is_some() {
                writeln!(f, "{}{}", padding, "|".blue().bold())?;
            }

            writeln!(f, "{}{} {} {}", padding, "|".blue().bold(), "help:".bold(), hint)?;
        } else if self.location.is_some() {
            writeln!(f, "{}{}", padding, "|".blue().bold())?;
        }

        Ok(())
    }
}

impl Debug for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// # Errors
/// This function will propagate errors from `write! and writeln!`
pub fn highlight_source<S: Into<String>>(
    f: &mut Formatter<'_>,
    source: S,
    location: &Location,
    padding: &str,
    level: LogLevel,
) -> fmt::Result {
    let source: String = source.into();

    for (idx, line) in source.lines().enumerate() {
        if !utils::range_contains(&location.lines, idx) {
            continue;
        }

        let line_number = (idx + 1).to_string().blue().bold();

        write!(f, "{line_number}{}{} ", &padding[1..], "|".blue().bold())?;

        if let Some(section) = &location.section {
            writeln!(
                f,
                "{}",
                match level {
                    LogLevel::Info | LogLevel::Debug | LogLevel::Trace => utils::bold_highlight(line, section),
                    _ => utils::highlight(line, section, level.into()),
                }
            )?;
        } else {
            writeln!(f, "{}", line.color(level).bold())?;
        }
    }

    Ok(())
}

#[macro_export]
macro_rules! set_app_name {
    () => {
        env::set_var("LOGGER_APP_NAME", env!("CARGO_PKG_NAME"))
    };
}
