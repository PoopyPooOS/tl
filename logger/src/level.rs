use std::fmt::Display;

use colored::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

impl From<LogLevel> for Color {
    fn from(val: LogLevel) -> Self {
        match val {
            LogLevel::Trace => Color::Magenta,
            LogLevel::Debug => Color::Green,
            LogLevel::Info => Color::Blue,
            LogLevel::Warning => Color::Yellow,
            LogLevel::Error => Color::BrightRed,
            LogLevel::Fatal => Color::Red,
        }
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogLevel::Trace => "trace",
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warning => "warning",
                LogLevel::Error => "error",
                LogLevel::Fatal => "fatal",
            }
        )
    }
}
