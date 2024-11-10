macro_rules! define_log_helper {
    ($name:ident, $make_name:ident, $level:ident) => {
        #[macro_export]
        macro_rules! $name {
            ($message:expr) => {{
                $crate::Log {
                    level: $crate::level::LogLevel::$level,
                    message: $message.to_string(),
                    location: None,
                    hint: None,
                }
                .output();
            }};
            ($message:expr, location: $location:expr) => {{
                $crate::Log {
                    level: $crate::level::LogLevel::$level,
                    message: $message.to_string(),
                    location: Some($location),
                    hint: None,
                }
                .output();
            }};
            ($message:expr, hint: $hint:expr) => {{
                $crate::Log {
                    level: $crate::level::LogLevel::$level,
                    message: $message.to_string(),
                    location: None,
                    hint: Some($hint.to_string()),
                }
                .output();
            }};
            ($message:expr, location: $location:expr, hint: $hint:expr) => {{
                $crate::Log {
                    level: $crate::level::LogLevel::$level,
                    message: $message.to_string(),
                    location: Some($location),
                    hint: Some($hint.to_string()),
                }
                .output();
            }};
            ($message:expr, hint: $hint:expr, location: $location:expr) => {{
                $crate::Log {
                    level: $crate::level::LogLevel::$level,
                    message: $message.to_string(),
                    location: Some($location),
                    hint: Some($hint.to_string()),
                }
                .output();
            }};
        }

        #[macro_export]
        macro_rules! $make_name {
            ($message:expr) => {{
                $crate::Log {
                    level: $crate::level::LogLevel::$level,
                    message: $message.to_string(),
                    location: None,
                    hint: None,
                }
            }};
            ($message:expr, location: $location:expr) => {{
                $crate::Log {
                    level: $crate::level::LogLevel::$level,
                    message: $message.to_string(),
                    location: Some($location),
                    hint: None,
                }
            }};
            ($message:expr, hint: $hint:expr) => {{
                $crate::Log {
                    level: $crate::level::LogLevel::$level,
                    message: $message.to_string(),
                    location: None,
                    hint: Some($hint.to_string()),
                }
            }};
            ($message:expr, location: $location:expr, hint: $hint:expr) => {{
                $crate::Log {
                    level: $crate::level::LogLevel::$level,
                    message: $message.to_string(),
                    location: Some($location),
                    hint: Some($hint.to_string()),
                }
            }};
            ($message:expr, hint: $hint:expr, location: $location:expr) => {{
                $crate::Log {
                    level: $crate::level::LogLevel::$level,
                    message: $message.to_string(),
                    location: Some($location),
                    hint: Some($hint.to_string()),
                }
            }};
        }
    };
}

define_log_helper!(trace, make_trace, Trace);
define_log_helper!(debug, make_debug, Debug);
define_log_helper!(info, make_info, Info);
define_log_helper!(warn, make_warn, Warning);
define_log_helper!(error, make_error, Error);
define_log_helper!(fatal, make_fatal, Fatal);
