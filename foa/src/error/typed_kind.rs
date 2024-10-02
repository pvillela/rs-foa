//===========================
// region:      --- TypedErrorKind

use std::{backtrace::Backtrace, marker::PhantomData};

use super::{BacktraceSpec, Error, ErrorTag, KindId};

#[derive(Debug)]
pub struct TypedErrorKind<T>
where
    T: std::error::Error + Send + Sync + 'static,
{
    kind_id: KindId,
    backtrace_spec: BacktraceSpec,
    tag: Option<&'static ErrorTag>,
    _t: PhantomData<T>,
}

impl<T> TypedErrorKind<T>
where
    T: std::error::Error + Send + Sync + 'static,
{
    pub const fn kind_id(&self) -> &KindId {
        &self.kind_id
    }

    pub const fn tag(&self) -> Option<&'static ErrorTag> {
        self.tag
    }

    pub const fn new(
        name: &'static str,
        backtrace_spec: BacktraceSpec,
        tag: Option<&'static ErrorTag>,
    ) -> Self {
        Self {
            kind_id: KindId(name),
            backtrace_spec,
            tag,
            _t: PhantomData,
        }
    }

    pub fn error(&'static self, payload: T) -> Error {
        let backtrace = match self.backtrace_spec {
            BacktraceSpec::Yes => Some(Backtrace::force_capture()),
            BacktraceSpec::No => None,
            BacktraceSpec::Env => Some(Backtrace::capture()),
        };

        Error::new(self.kind_id(), self.tag, payload, backtrace)
    }
}

// endregion:   --- TypedErrorKind

#[cfg(test)]
mod test {
    use super::TypedErrorKind;
    use crate::error::{BacktraceSpec, TrivialError};

    const FOO_ERROR: TypedErrorKind<TrivialError> =
        TypedErrorKind::new("FOO_ERROR", BacktraceSpec::Env, None);

    #[test]
    fn test() {
        let err = FOO_ERROR.error(TrivialError(""));
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "".to_string());
    }
}
