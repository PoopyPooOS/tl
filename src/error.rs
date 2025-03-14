use logger::Location;
use std::fmt::{Debug, Display};

#[derive(Debug, PartialEq)]
pub struct Error<T: Debug + PartialEq> {
    pub error_type: T,
    pub location: Option<Location>,
}

impl<T: Debug + PartialEq> std::error::Error for Error<T> {}

impl<T: Debug + PartialEq> Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.error_type)
    }
}

impl<T: Debug + PartialEq> Error<T> {
    pub fn new(error_type: T, location: Option<Location>) -> Self {
        Self {
            error_type,
            location,
        }
    }
}
