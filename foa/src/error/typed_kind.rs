//===========================
// region:      --- TypedErrorKind

use std::{backtrace::Backtrace, marker::PhantomData};

use super::{BacktraceSpec, Error, ErrorTag, KindId, StdBoxError};

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
}

impl<T> TypedErrorKind<T>
where
    T: std::error::Error + Send + Sync + 'static,
{
    pub fn error(&'static self, payload: T) -> Error {
        let backtrace = match self.backtrace_spec {
            BacktraceSpec::Yes => Some(Backtrace::force_capture()),
            BacktraceSpec::No => None,
            BacktraceSpec::Env => Some(Backtrace::capture()),
        };

        Error::new(
            self.kind_id(),
            self.tag,
            StdBoxError::new(payload),
            backtrace,
        )
    }
}

// endregion:   --- TypedErrorKind

#[cfg(test)]
mod test {
    use crate::error::BacktraceSpec;

    use super::TypedErrorKind;
    use std::fmt::{Debug, Display};

    #[derive(Debug)]
    struct Dummy;

    impl Display for Dummy {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(&self, f)
        }
    }

    impl std::error::Error for Dummy {}

    const FOO_ERROR: TypedErrorKind<Dummy> =
        TypedErrorKind::new("FOO_ERROR", BacktraceSpec::Env, None);

    #[test]
    fn test() {
        let err = FOO_ERROR.error(Dummy);
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), Dummy.to_string());
    }
}
