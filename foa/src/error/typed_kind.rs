use super::{BacktraceSpec, Error, KindId, Tag};
use std::{backtrace::Backtrace, marker::PhantomData};

#[derive(Debug)]
pub struct TypedKind<T>
where
    T: std::error::Error + Send + Sync + 'static,
{
    kind_id: KindId,
    backtrace_spec: BacktraceSpec,
    tag: Option<&'static Tag>,
    _t: PhantomData<T>,
}

impl<T> TypedKind<T>
where
    T: std::error::Error + Send + Sync + 'static,
{
    pub const fn kind_id(&self) -> &KindId {
        &self.kind_id
    }

    pub const fn tag(&self) -> Option<&'static Tag> {
        self.tag
    }

    pub const fn new(
        name: &'static str,
        backtrace_spec: BacktraceSpec,
        tag: Option<&'static Tag>,
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
            BacktraceSpec::Yes => Backtrace::force_capture(),
            BacktraceSpec::No => Backtrace::disabled(),
            BacktraceSpec::Env => Backtrace::capture(),
        };

        Error::new(self.kind_id(), self.tag, payload, backtrace)
    }
}

#[cfg(test)]
mod test {
    use super::TypedKind;
    use crate::error::{BacktraceSpec, TrivialError};

    static FOO_ERROR: TypedKind<TrivialError> =
        TypedKind::new("FOO_ERROR", BacktraceSpec::Env, None);

    #[test]
    fn test() {
        let err = FOO_ERROR.error(TrivialError(""));
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "".to_string());
    }
}
