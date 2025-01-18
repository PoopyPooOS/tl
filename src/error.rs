use logger::Location;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct Error<T: Debug> {
    pub error_type: T,
    pub location: Option<Location>,
}

impl<T: Debug> std::error::Error for Error<T> {}

impl<T: Debug> Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.error_type)
    }
}

impl<T: Debug> Error<T> {
    #[must_use]
    pub fn new(error_type: T, location: Option<Location>) -> Self {
        Self {
            error_type,
            location,
        }
    }
}
