use super::{BacktraceSpec, Error, KindId, Payload, StdBoxError, Tag};
use std::{backtrace::Backtrace, error::Error as StdError, marker::PhantomData};

#[derive(Debug)]
pub struct TypedKind<T, const HASCAUSE: bool> {
    kind_id: KindId,
    msg: Option<&'static str>,
    backtrace_spec: BacktraceSpec,
    tag: &'static Tag,
    _t: PhantomData<T>,
}

impl<T: Payload, const HASCAUSE: bool> TypedKind<T, HASCAUSE> {
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

    pub const fn new(
        name: &'static str,
        msg: Option<&'static str>,
        backtrace_spec: BacktraceSpec,
        tag: &'static Tag,
    ) -> Self {
        Self {
            kind_id: KindId(name),
            msg,
            tag,
            backtrace_spec,
            _t: PhantomData,
        }
    }

    fn error_priv(&'static self, payload: T, source: Option<StdBoxError>) -> Error {
        let backtrace = match self.backtrace_spec {
            BacktraceSpec::Yes => Backtrace::force_capture(),
            BacktraceSpec::No => Backtrace::disabled(),
            BacktraceSpec::Env => Backtrace::capture(),
        };

        Error::new(
            &self.kind_id,
            self.msg(),
            self.tag,
            payload,
            source,
            backtrace,
        )
    }
}

impl<T: Payload> TypedKind<T, false> {
    pub fn error(&'static self, payload: T) -> Error {
        self.error_priv(payload, None)
    }
}

impl<T: Payload> TypedKind<T, true> {
    pub fn error(
        &'static self,
        payload: T,
        source: impl StdError + Send + Sync + 'static,
    ) -> Error {
        self.error_priv(payload, Some(StdBoxError::new(source)))
    }
}

#[cfg(test)]
mod test {
    use super::TypedKind;
    use crate::error::{BacktraceSpec, Tag};

    static FOO_TAG: Tag = Tag("FOO");

    static FOO_ERROR: TypedKind<String, false> =
        TypedKind::new("FOO_ERROR", None, BacktraceSpec::Env, &FOO_TAG);

    #[test]
    fn test() {
        let err = FOO_ERROR.error("dummy payload".to_owned());
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "".to_string());
    }
}
