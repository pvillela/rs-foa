use super::{extract_boxed_error, JserBoxError, JserError, StdBoxError};
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

    pub fn payload_ref(&self) -> &Box<(dyn std::error::Error + Send + Sync + 'static)> {
        &self.payload.0
    }

    pub fn typed_payload_ref<T: StdError + 'static>(&self) -> Option<&T> {
        self.payload.0.downcast_ref::<T>()
    }

    /// Returns `self`'s typed payload.
    ///
    /// As this method consumes `self`, if you also need access to other [`Error`] fields then convert
    /// to an [`ErrorExp`] using `self.into()` instead.
    pub fn typed_payload<T: StdError + 'static>(self) -> Result<T> {
        // return early if type doesn't match
        let typed_ref = self.payload.0.downcast_ref::<T>();
        match typed_ref {
            None => Err(self),
            Some(_) => {
                let res = extract_boxed_error::<T>(self.payload.0);
                match res {
                    Ok(payload) => Ok(payload),
                    Err(_) => unreachable!("downcast previously confirmed"),
                }
            }
        }
    }

    /// If the payload is of type `T`, returns `Err(f(payload))`;
    /// otherwise, returns `Ok(err)` where `err` is identical to self.
    /// This unusual signature facilitates chaining of calls of this method with different types.
    ///
    /// As this method consumes `self`, if you also need access to other [`Error`] fields then convert
    /// use [`Error::with_errorexp`] instead.
    ///
    /// # Example
    /// ```
    /// use foa::Error;
    /// use std::fmt::Debug;
    ///
    /// fn process_error<T1: std::error::Error + 'static + Debug, T2: std::error::Error + 'static + Debug>(
    ///     err: Error,
    /// ) -> std::result::Result<Error, ()> {
    ///     err.with_typed_payload::<T1, ()>(|pld| println!("payload type was `T1`: {pld:?}"))?
    ///         .with_typed_payload::<T2, ()>(|pld| println!("payload type was `T2: {pld:?}`"))
    /// }
    /// ```
    pub fn with_typed_payload<T: StdError + 'static, U>(
        self,
        f: impl FnOnce(T) -> U,
    ) -> std::result::Result<Error, U> {
        let res = self.typed_payload::<T>();
        match res {
            Ok(payload) => Err(f(payload)),
            Err(err) => Ok(err),
        }
    }

    pub fn backtrace(&self) -> Option<&Backtrace> {
        match &self.backtrace {
            Some(backtrace) => Some(backtrace),
            None => None,
        }
    }

    /// If the payload is of type `T`, returns `Err(f(error_exp))` where `error_exp` is the
    /// [`ErrorExp`] instance obtained from `self`;
    /// otherwise, returns `Ok(err)` where `err` is identical to self.
    /// This unusual signature facilitates chaining of calls of this method with different types.
    ///
    /// # Example
    /// ```
    /// use foa::Error;
    /// use std::fmt::Debug;
    ///
    /// fn process_error<T1: std::error::Error + 'static + Debug, T2: std::error::Error + 'static + Debug>(
    ///     err: Error,
    /// ) -> std::result::Result<Error, ()> {
    ///     err.with_errorexp::<T1, ()>(|ee| println!("payload type was `T1`: {ee:?}"))?
    ///         .with_errorexp::<T2, ()>(|ee| println!("payload type was `T2`: {ee:?}"))
    /// }
    /// ```
    pub fn with_errorexp<T: StdError + 'static, U>(
        self,
        f: impl FnOnce(ErrorExp<T>) -> U,
    ) -> std::result::Result<Error, U> {
        let res: Result<ErrorExp<T>> = self.into();
        match res {
            Ok(error_exp) => Err(f(error_exp)),
            Err(err) => Ok(err),
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

// endregion:   --- Error

//===========================
// region:      --- ErrorExp

/// Struct with the same fields as [`Error`] but where the payload is a type `T` rather than a boxed error.
#[derive(Debug, Serialize)]
pub struct ErrorExp<T> {
    pub kind_id: &'static KindId,
    pub tag: Option<&'static ErrorTag>,
    pub payload: T,
    #[serde(skip_serializing)]
    pub backtrace: Option<Backtrace>,
}

impl<T: StdError + 'static> From<Error> for Result<ErrorExp<T>> {
    fn from(value: Error) -> Self {
        // return early if type doesn't match
        let typed_ref = value.payload.0.downcast_ref::<T>();
        match typed_ref {
            None => Err(value),
            Some(_) => {
                let res = extract_boxed_error::<T>(value.payload.0);
                match res {
                    Ok(payload) => Ok(ErrorExp {
                        kind_id: value.kind_id,
                        tag: value.tag,
                        payload,
                        backtrace: value.backtrace,
                    }),
                    Err(_) => unreachable!("downcast previously confirmed"),
                }
            }
        }
    }
}

// endregion:   --- ErrorExp

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::{PropsError, PropsErrorKind, TrivialError};

    const FOO_ERROR: PropsErrorKind<1, false> = PropsErrorKind::with_prop_names(
        "FOO_ERROR",
        Some("foo message: {xyz}"),
        ["xyz"],
        BacktraceSpec::Env,
        None,
    );

    fn make_payload_error_pair() -> (PropsError, Error) {
        fn make_payload() -> PropsError {
            PropsError {
                msg: FOO_ERROR.msg.unwrap(),
                props: vec![(FOO_ERROR.prop_names[0].into(), "hi there!".into())],
                source: None,
            }
        }

        let err = Error::new(&FOO_ERROR.kind_id(), FOO_ERROR.tag(), make_payload(), None);

        (make_payload(), err)
    }

    #[test]
    fn test_extract_payload() {
        let (payload, err) = make_payload_error_pair();

        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message: hi there!");

        let payload_ext: PropsError = err.typed_payload().unwrap();

        assert_eq!(payload.msg, payload_ext.msg);
        assert_eq!(payload.props, payload_ext.props);
    }

    #[test]
    fn test_with_typed_payload() {
        let (payload, err) = make_payload_error_pair();

        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message: hi there!");

        let run_assertions = || -> std::result::Result<Error, ()> {
            err.with_typed_payload::<TrivialError, _>(|_| unreachable!())?
                .with_typed_payload::<PropsError, _>(|payload_ext| {
                    assert_eq!(payload.msg, payload_ext.msg);
                    assert_eq!(payload.props, payload_ext.props);
                })?
                .with_typed_payload::<TrivialError, _>(|_| unreachable!("again"))
        };
        assert!(run_assertions().is_err());
    }

    #[test]
    fn test_with_errorexp() {
        let (payload, err) = make_payload_error_pair();
        let (_, err1) = make_payload_error_pair();

        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message: hi there!");

        let run_assertions = || -> std::result::Result<Error, ()> {
            err.with_errorexp::<TrivialError, _>(|_| unreachable!())?
                .with_errorexp::<PropsError, _>(|ee| {
                    assert_eq!(payload.msg, ee.payload.msg);
                    assert_eq!(payload.props, ee.payload.props);
                    assert_eq!(err1.kind_id(), ee.kind_id);
                    assert_eq!(err1.tag(), ee.tag);
                })?
                .with_errorexp::<TrivialError, _>(|_| unreachable!("again"))
        };
        assert!(run_assertions().is_err());
    }
}
