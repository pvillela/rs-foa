use super::{Error, ErrorTag, KindId, StdBoxError};
use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
};

//===========================
// region:      --- BasicError

#[derive(Debug)]
pub struct BasicError {
    msg: &'static str,
    source: Option<StdBoxError>,
}

impl Display for BasicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg)
    }
}

impl std::error::Error for BasicError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.source {
            Some(e) => Some(e.as_dyn_std_error()),
            None => None,
        }
    }
}

// endregion:   --- BasicError

//===========================
// region:      --- BasicErrorKind

#[derive(Debug)]
pub struct BasicErrorKind<const HASCAUSE: bool> {
    kind_id: KindId,
    msg: Option<&'static str>,
    tag: Option<&'static ErrorTag>,
}

impl<const HASCAUSE: bool> BasicErrorKind<HASCAUSE> {
    pub const fn kind_id(&self) -> &KindId {
        &self.kind_id
    }

    pub const fn tag(&self) -> Option<&'static ErrorTag> {
        self.tag
    }

    pub const fn new(
        name: &'static str,
        msg: Option<&'static str>,
        tag: Option<&'static ErrorTag>,
    ) -> Self {
        Self {
            kind_id: KindId(name),
            msg,
            tag,
        }
    }

    fn new_error_priv(&'static self, cause: Option<StdBoxError>) -> Error {
        let msg = match self.msg {
            Some(msg) => msg,
            None => self.kind_id.0,
        };
        let payload = BasicError { msg, source: cause };
        Error::new(self.kind_id(), self.tag, payload)
    }
}

impl BasicErrorKind<false> {
    pub fn new_error(&'static self) -> Error {
        self.new_error_priv(None)
    }
}

impl BasicErrorKind<true> {
    pub fn new_error(&'static self, cause: impl StdError + Send + Sync + 'static) -> Error {
        self.new_error_priv(Some(StdBoxError::new(cause)))
    }
}

// endregion:   --- BasicErrorKind

#[cfg(test)]
mod test {
    use super::BasicErrorKind;

    const FOO_ERROR: BasicErrorKind<false> = BasicErrorKind::new("FOO_ERROR", None, None);

    #[test]
    fn test() {
        let err = FOO_ERROR.new_error();
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "FOO_ERROR");
    }
}
