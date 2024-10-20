use super::{BacktraceSpec, BasicKind, Error, KindId, Tag, UNEXPECTED_TAG};
use serde::Serialize;
use std::fmt::{Debug, Display};

/// Very simple error that simply encapsulates a `&static str`. Should only be used for tests and examples,
/// not recommended for production applications or libraries.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TrivialError(pub &'static str);

impl Display for TrivialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for TrivialError {}

/// Error kind instance that can be used to wrap unexpected errors.
pub static UNEXPECTED_ERROR: BasicKind<true> =
    BasicKind::new("UNEXPECTED_ERROR", None, &UNEXPECTED_TAG).with_backtrace(BacktraceSpec::Yes);

/// Supports the replacement of an existing [`Error`] intances's `kind_id`, `msg`, and `tag`.
///
/// See [`Self::new`] and [`Self::transmute`].
pub struct TransmuterKind {
    kind_id: KindId,
    msg: Option<&'static str>,
    tag: &'static Tag,
}

impl TransmuterKind {
    pub const fn kind_id(&self) -> &KindId {
        &self.kind_id
    }

    pub const fn msg(&self) -> &'static str {
        match self.msg {
            Some(msg) => msg,
            None => self.kind_id.0,
        }
    }

    pub const fn tag(&self) -> &'static Tag {
        self.tag
    }

    pub const fn new(name: &'static str, msg: Option<&'static str>, tag: &'static Tag) -> Self {
        Self {
            kind_id: KindId(name),
            msg,
            tag,
        }
    }

    pub fn transmute(&'static self, err: Error) -> Error {
        Error {
            kind_id: &self.kind_id,
            msg: self.msg().into(),
            tag: self.tag,
            props: err.props,
            payload: err.payload,
            source: err.source,
            backtrace: err.backtrace,
            ref_id: err.ref_id,
        }
    }
}
