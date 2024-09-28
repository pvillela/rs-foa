use super::{JserBoxError, JserError, StdBoxError};
use serde::Serialize;
use std::{
    backtrace::Backtrace,
    error::Error as StdError,
    fmt::{Debug, Display},
};

pub const TRUNC: usize = 8;

//===========================
// region:      --- ErrorTag

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ErrorTag(pub &'static str);

// endregion:   --- ErrorTag

//===========================
// region:      --- Backtrace

/// Specifies different backtrace generation modes.
#[derive(Debug)]
pub enum BacktraceSpec {
    /// A backtrace is always generated
    Yes,
    /// A backtrace is never generated
    No,
    /// Backtrace generation is based on environment variables as per
    /// [`std::backtrace::Backtrace`](https://doc.rust-lang.org/std/backtrace/struct.Backtrace.html).
    Env,
}

// endregion:   --- Backtrace

//===========================
// region:      --- KindId

#[derive(Serialize)]
pub struct KindId(pub &'static str);

impl KindId {
    fn address(&self) -> usize {
        self as *const Self as usize
    }
}

impl Debug for KindId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            f.write_fmt(format_args!("KindId({}, {})", self.0, self.address()))
        } else {
            f.write_fmt(format_args!("KindId({})", self.0))
        }
    }
}

impl PartialEq for KindId {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Eq for KindId {}

// endregion:   --- KindId

//===========================
// region:      --- Error

#[derive(Debug, Serialize)]
pub struct Error {
    kind_id: &'static KindId,
    tag: Option<&'static ErrorTag>,
    payload: StdBoxError,
    #[serde(skip_serializing)]
    backtrace: Option<Backtrace>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(
        kind_id: &'static KindId,
        tag: Option<&'static ErrorTag>,
        payload: impl StdError + Send + Sync + 'static,
        backtrace: Option<Backtrace>,
    ) -> Self {
        Self {
            kind_id,
            tag,
            payload: StdBoxError::new(payload),
            backtrace,
        }
    }

    pub fn has_kind(&self, kind: &'static KindId) -> bool {
        self.kind_id == kind
    }

    pub fn kind_id(&self) -> &'static KindId {
        self.kind_id
    }

    pub fn tag(&self) -> Option<&'static ErrorTag> {
        self.tag
    }

    pub fn payload(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        &self.payload
    }

    pub fn payload_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        &mut self.payload
    }

    pub fn backtrace(&self) -> Option<&Backtrace> {
        match &self.backtrace {
            Some(backtrace) => Some(backtrace),
            None => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.payload, f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.payload.source()
    }
}

impl From<Error> for Box<dyn JserError> {
    fn from(value: Error) -> Self {
        Box::new(value)
    }
}

impl From<Error> for JserBoxError {
    fn from(value: Error) -> Self {
        JserBoxError::new(value)
    }
}

// endregion:   --- ErrorKind
