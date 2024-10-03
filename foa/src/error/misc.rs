use super::{BacktraceSpec, BasicErrorKind, Error, ErrorTag, KindId, UNEXPECTED_ERROR_TAG};
use serde::Serialize;
use std::{
    backtrace::Backtrace,
    error::Error as StdError,
    fmt::{Debug, Display},
};

/// Very simple error that simply encapsulates a `&static str`. Should only be used for tests and examples,
/// not recommended for production applications or libraries.
#[derive(Debug, Serialize, PartialEq)]
pub struct TrivialError(pub &'static str);

impl Display for TrivialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for TrivialError {}

#[derive(Debug)]
pub struct UnexpectedErrorKind {
    kind_id: KindId,
}

impl UnexpectedErrorKind {
    pub const fn kind_id(&self) -> &KindId {
        &self.kind_id
    }

    pub fn tag(&self) -> Option<&'static ErrorTag> {
        Some(&UNEXPECTED_ERROR_TAG)
    }

    pub const fn new(name: &'static str) -> Self {
        Self {
            kind_id: KindId(name),
        }
    }

    pub fn error<T: StdError + Send + Sync + 'static>(&'static self, payload: T) -> Error {
        let backtrace = Some(Backtrace::force_capture());
        let internal_payload = UNEXPECTED_ERROR_PAYLOAD.error(payload);
        Error::new(self.kind_id(), self.tag(), internal_payload, backtrace)
    }
}

static UNEXPECTED_ERROR_PAYLOAD: BasicErrorKind<true> =
    BasicErrorKind::new("UNEXPECTED_ERROR", None, BacktraceSpec::No, None);

pub static UNEXPECTED_ERROR: UnexpectedErrorKind = UnexpectedErrorKind::new("UNEXPECTED_ERROR");
