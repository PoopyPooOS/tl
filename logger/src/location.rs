use std::{fmt::Display, fs, io, ops::RangeInclusive, path::PathBuf};

#[derive(Debug)]
pub struct Location {
    pub path: Option<PathBuf>,
    /// Optional field for builtin sources
    pub text: Option<String>,
    pub lines: RangeInclusive<usize>,
    pub section: Option<RangeInclusive<usize>>,
}

impl Location {
    pub fn new(path: impl Into<Option<PathBuf>>, lines: RangeInclusive<usize>) -> Self {
        Self {
            path: path.into(),
            text: None,
            lines,
            section: None,
        }
    }

    pub fn new_with_section(path: impl Into<Option<PathBuf>>, lines: RangeInclusive<usize>, section: RangeInclusive<usize>) -> Self {
        Self::new(path, lines).section(section)
    }

    pub fn section(mut self, section: RangeInclusive<usize>) -> Self {
        self.section = Some(section);
        self
    }

    /// # Errors
    ///
    /// This function will propagate errors from `std::fs::read_to_string`
    pub fn read(&self) -> io::Result<String> {
        if let Some(path) = &self.path {
            fs::read_to_string(path)
        } else {
            if let Some(text) = &self.text {
                return Ok(text.clone());
            } else {
                return Err(io::Error::new(io::ErrorKind::NotFound, "Source not found"));
            }
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Add 1 to line and column because humans (and code editors) use 1 based indexing
        if let Some(path) = &self.path {
            write!(
                f,
                "{}:{}{}",
                path.display(),
                self.lines.end() + 1,
                if let Some(section) = &self.section {
                    format!(":{}", section.end() + 1)
                } else {
                    String::new()
                }
            )
        } else {
            write!(
                f,
                "unknown:{}{}",
                self.lines.end() + 1,
                if let Some(section) = &self.section {
                    format!(":{}", section.end() + 1)
                } else {
                    String::new()
                }
            )
        }
    }
}
