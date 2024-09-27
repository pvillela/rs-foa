//===========================
// region:      --- TypedErrorKind

use std::marker::PhantomData;

use super::{Error, ErrorTag, KindId, StdBoxError};

#[derive(Debug)]
pub struct TypedErrorKind<T>
where
    T: std::error::Error + Send + Sync + 'static,
{
    kind_id: KindId,
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

    pub const fn new(name: &'static str, tag: Option<&'static ErrorTag>) -> Self {
        Self {
            kind_id: KindId(name),
            tag,
            _t: PhantomData,
        }
    }
}

impl<T> TypedErrorKind<T>
where
    T: std::error::Error + Send + Sync + 'static,
{
    pub fn new_error(&'static self, payload: T) -> Error {
        Error::new(self.kind_id(), self.tag, StdBoxError::new(payload))
    }
}

// endregion:   --- TypedErrorKind

#[cfg(test)]
mod test {
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

    const FOO_ERROR: TypedErrorKind<Dummy> = TypedErrorKind::new("FOO_ERROR", None);

    #[test]
    fn test() {
        let err = FOO_ERROR.new_error(Dummy);
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), Dummy.to_string());
    }
}
