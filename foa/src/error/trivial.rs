use std::fmt::{Debug, Display};

/// Very simple error that simply encapsulates a `&static str`. Should only be used for tests and examples,
/// not recommended for production applications or libraries.
#[derive(Debug)]
pub struct TrivialError(pub &'static str);

impl Display for TrivialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for TrivialError {}
