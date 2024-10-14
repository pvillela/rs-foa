use super::{BoxPayload, Fmt, JserBoxError, Payload, StdBoxError, WithBacktrace};
use crate::error::static_str::StaticStr;
use crate::error::utils::StringSpec;
use crate::nodebug::NoDebug;
use serde::Serialize;
use std::{
    backtrace::Backtrace,
    collections::BTreeMap,
    error::Error as StdError,
    fmt::{Debug, Display},
};

pub const TRUNC: usize = 8;

//===========================
// region:      --- Tag

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Tag(pub &'static str);

// endregion:   --- Tag

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

#[derive(Debug)]
pub struct Error {
    pub(crate) kind_id: &'static KindId,
    pub(super) msg: StaticStr,
    pub(crate) tag: &'static Tag,
    pub(crate) payload: BoxPayload,
    pub(crate) source: Option<StdBoxError>,
    pub(crate) backtrace: NoDebug<Backtrace>,
}

pub type Result<T> = std::result::Result<T, Error>;

pub type ReverseResult<T> = std::result::Result<Error, T>;

impl Error {
    pub fn new(
        kind_id: &'static KindId,
        msg: &'static str,
        tag: &'static Tag,
        payload: impl Payload,
        source: Option<StdBoxError>,
        backtrace: Backtrace,
    ) -> Self {
        let source = match source {
            Some(e) => Some(StdBoxError::new(e)),
            None => None,
        };
        Self {
            kind_id,
            msg: msg.into(),
            tag,
            payload: BoxPayload::new(payload),
            source,
            backtrace: NoDebug(backtrace),
        }
    }

    pub fn with_msg_string(
        kind_id: &'static KindId,
        msg: String,
        tag: &'static Tag,
        payload: impl Payload,
        source: Option<StdBoxError>,
        backtrace: Backtrace,
    ) -> Self {
        let source = match source {
            Some(e) => Some(StdBoxError::new(e)),
            None => None,
        };
        Self {
            kind_id,
            msg: msg.into(),
            tag,
            payload: BoxPayload::new(payload),
            source,
            backtrace: NoDebug(backtrace),
        }
    }

    pub fn has_kind(&self, kind: &'static KindId) -> bool {
        self.kind_id == kind
    }

    pub fn kind_id(&self) -> &'static KindId {
        self.kind_id
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn tag(&self) -> &'static Tag {
        self.tag
    }

    pub fn payload_ref(&self) -> &dyn Payload {
        self.payload.as_ref()
    }

    pub fn typed_payload_ref<T: Payload>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }

    /// If the payload is of type `T`, returns `Ok(payload)` , otherwise returns `Err(self)`.
    ///
    /// As this method consumes `self`, if you also need access to other [`Error`] fields then convert
    /// to an [`ErrorExt`] using [`Self::into_ErrorExt`] instead.
    pub fn typed_payload<T: Payload>(self) -> Result<Box<T>> {
        if self.payload.is::<T>() {
            let res = self.payload.downcast::<T>();
            match res {
                Ok(payload) => Ok(payload),
                Err(_) => unreachable!("downcast previously confirmed"),
            }
        } else {
            Err(self)
        }
    }

    /// If the payload is of type `T`, returns `Err(f(payload))`; otherwise, returns `Ok(self)`
    /// This unusual signature facilitates chaining of calls of this method for different types.
    ///
    /// As this method consumes `self`, if you also need access to other [`Error`] fields then convert
    /// use [`Error::with_ErrorExt`] instead.
    ///
    /// # Example
    /// ```
    /// use foa::{Error, Result, ReverseResult, error::{Payload, swap_result}};
    /// use std::fmt::Debug;
    ///
    /// fn process_error<T1: Payload, T2: Payload>(
    ///     err: Error,
    /// ) -> Result<()> {
    ///     swap_result(|| -> ReverseResult<()> {
    ///         err.with_typed_payload::<T1, ()>(|pld| println!("payload type was `T1`: {pld:?}"))?
    ///             .with_typed_payload::<T2, ()>(|pld| println!("payload type was `T2: {pld:?}`"))
    ///     })
    /// }
    /// ```
    pub fn with_typed_payload<T: Payload, U>(
        self,
        f: impl FnOnce(Box<T>) -> U,
    ) -> ReverseResult<U> {
        let res = self.typed_payload::<T>();
        match res {
            Ok(payload) => Err(f(payload)),
            Err(err) => Ok(err),
        }
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    /// If the payload is of type `T`, returns `Ok(error_ext)`, where `error_ext` is the
    /// [`ErrorExt`] instance obtained from `self`; otherwise returns `Err(self)`.
    pub fn into_errorext<T: Payload>(self) -> Result<ErrorExt<T>> {
        if self.payload.is::<T>() {
            let res = self.payload.downcast::<T>();
            match res {
                Ok(payload) => Ok(ErrorExt {
                    kind_id: self.kind_id,
                    msg: self.msg,
                    tag: self.tag,
                    payload,
                    source: self.source,
                    backtrace: self.backtrace,
                }),
                Err(_) => unreachable!("downcast previously confirmed"),
            }
        } else {
            Err(self)
        }
    }

    /// If the payload is of type `T`, returns `Err(f(error_ext))` where `error_ext` is the
    /// [`ErrorExt`] instance obtained from `self`; otherwise, returns `Ok(self)`.
    /// This unusual signature facilitates chaining of calls of this method for different types.
    ///
    /// # Example
    /// ```
    /// use foa::{Error, Result, ReverseResult, error::{Payload, swap_result}};
    /// use std::fmt::Debug;
    ///
    /// fn process_error<T1: Payload, T2: Payload>(
    ///     err: Error,
    /// ) -> Result<()> {
    ///     swap_result(|| -> ReverseResult<()> {
    ///         err.with_ErrorExt::<T1, ()>(|ee| println!("payload type was `T1`: {ee:?}"))?
    ///             .with_ErrorExt::<T2, ()>(|ee| println!("payload type was `T2`: {ee:?}"))
    ///     })
    /// }
    /// ```
    pub fn with_errorext<T: Payload, U>(
        self,
        f: impl FnOnce(ErrorExt<T>) -> U,
    ) -> ReverseResult<U> {
        let res = self.into_errorext::<T>();
        match res {
            Ok(error_ext) => Err(f(error_ext)),
            Err(err) => Ok(err),
        }
    }

    pub fn to_sererror<const N: usize>(&self, str_specs: [StringSpec; N]) -> SerError {
        let fmt = Fmt(self);
        let other = str_specs
            .into_iter()
            .map(|spec| fmt.speced_string_tuple(&spec))
            .collect::<BTreeMap<&'static str, String>>();
        SerError {
            kind_id: self.kind_id,
            msg: self.msg.clone(),
            tag: self.tag,
            other,
        }
    }

    pub fn as_fmt(&self) -> Fmt<'_, Self> {
        Fmt(self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.source {
            Some(e) => Some(e.as_dyn_std_error()),
            None => None,
        }
    }
}

impl WithBacktrace for Error {
    fn backtrace(&self) -> &Backtrace {
        Self::backtrace(&self)
    }
}

// endregion:   --- Error

//===========================
// region:      --- ErrorExt

/// Struct with the same fields as [`Error`] but where the payload is a type `T` rather than a boxed error.
#[derive(Debug)]
pub struct ErrorExt<T> {
    kind_id: &'static KindId,
    msg: StaticStr,
    tag: &'static Tag,
    payload: Box<T>,
    source: Option<StdBoxError>,
    backtrace: NoDebug<Backtrace>,
}

impl<T: Payload> ErrorExt<T> {
    pub fn has_kind(&self, kind: &'static KindId) -> bool {
        self.kind_id == kind
    }

    pub fn kind_id(&self) -> &'static KindId {
        self.kind_id
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn tag(&self) -> &'static Tag {
        self.tag
    }

    pub fn payload_ref(&self) -> &T {
        &self.payload
    }

    pub fn payload(self) -> Box<T> {
        self.payload
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    pub fn as_fmt(&self) -> Fmt<'_, Self> {
        Fmt(self)
    }
}

impl<T: Payload + Serialize> ErrorExt<T> {
    pub fn into_sererrorext<const N: usize>(self, str_specs: [StringSpec; N]) -> SerErrorExt<T> {
        let fmt = Fmt(&self);
        let other = str_specs
            .into_iter()
            .map(|spec| fmt.speced_string_tuple(&spec))
            .collect::<BTreeMap<&'static str, String>>();
        SerErrorExt {
            kind_id: self.kind_id,
            msg: self.msg,
            tag: self.tag,
            payload: self.payload,
            other,
        }
    }
}

impl<T: Payload> Display for ErrorExt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl<T: Payload> StdError for ErrorExt<T> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.source {
            Some(e) => Some(e.as_dyn_std_error()),
            None => None,
        }
    }
}

impl<T> WithBacktrace for ErrorExt<T> {
    fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl<T: Payload> From<Error> for Result<ErrorExt<T>> {
    fn from(value: Error) -> Self {
        value.into_errorext()
    }
}

// endregion:   --- ErrorExt

//===========================
// region:      --- SerError, SerErrorExt

#[derive(Debug, Serialize)]
pub struct SerError {
    kind_id: &'static KindId,
    msg: StaticStr,
    tag: &'static Tag,
    other: BTreeMap<&'static str, String>,
}

impl SerError {
    pub fn kind_id(&self) -> &'static KindId {
        self.kind_id
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn tag(&self) -> &'static Tag {
        self.tag
    }

    pub fn other(&self) -> &BTreeMap<&'static str, String> {
        &self.other
    }
}

impl Display for SerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl StdError for SerError {}

impl From<SerError> for JserBoxError {
    fn from(value: SerError) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Serialize)]
pub struct SerErrorExt<T: Payload> {
    kind_id: &'static KindId,
    msg: StaticStr,
    tag: &'static Tag,
    pub(super) payload: Box<T>,
    other: BTreeMap<&'static str, String>,
}

impl<T: Payload> SerErrorExt<T> {
    pub fn kind_id(&self) -> &'static KindId {
        self.kind_id
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn tag(&self) -> &'static Tag {
        self.tag
    }

    pub fn payload_ref(&self) -> &T {
        &self.payload
    }

    pub fn payload(self) -> Box<T> {
        self.payload
    }

    pub fn other(&self) -> &BTreeMap<&'static str, String> {
        &self.other
    }
}

impl<T: Payload> Display for SerErrorExt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl<T: Payload> StdError for SerErrorExt<T> {}

impl<T: Payload + Serialize> From<SerErrorExt<T>> for JserBoxError {
    fn from(value: SerErrorExt<T>) -> Self {
        Self::new(value)
    }
}

// endregion:   --- SerError, SerErrorExt

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        error::{swap_result, Props, PropsKind, ReverseResult, TrivialError},
        validation::validc::VALIDATION_ERROR,
    };
    use valid::{constraint::Bound, Validate, ValidationError};

    static FOO_TAG: Tag = Tag("FOO");

    static FOO_ERROR: PropsKind<1, false> = PropsKind::with_prop_names(
        "FOO_ERROR",
        Some("foo message: {xyz}"),
        ["xyz"],
        BacktraceSpec::Env,
        &FOO_TAG,
    );

    fn make_payload_error_pair() -> (Props, Error) {
        fn make_payload() -> Props {
            Props(vec![(FOO_ERROR.prop_names[0].into(), "hi there!".into())])
        }

        let err = Error::new(
            FOO_ERROR.kind_id(),
            FOO_ERROR.msg().into(),
            FOO_ERROR.tag(),
            make_payload(),
            None,
            Backtrace::disabled(),
        );

        (make_payload(), err)
    }

    #[test]
    fn test_extract_payload() {
        let (payload, err) = make_payload_error_pair();

        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message: {xyz}");

        let payload_ext = err.typed_payload::<Props>().unwrap();
        assert_eq!(payload.0, payload_ext.0);
    }

    #[test]
    fn test_with_typed_payload() {
        let (payload, err) = make_payload_error_pair();

        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message: {xyz}");

        let res = swap_result(|| -> ReverseResult<()> {
            err.with_typed_payload::<TrivialError, _>(|_| unreachable!())?
                .with_typed_payload::<Props, _>(|payload_ext| {
                    assert_eq!(payload.0, payload_ext.0);
                })?
                .with_typed_payload::<TrivialError, _>(|_| unreachable!("again"))
        });
        assert!(res.is_ok());
    }

    #[test]
    fn test_with_errorext() {
        let (payload, err) = make_payload_error_pair();
        let (_, err1) = make_payload_error_pair();

        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message: {xyz}");

        let res = swap_result(|| -> std::result::Result<Error, ()> {
            err.with_errorext::<TrivialError, _>(|_| unreachable!())?
                .with_errorext::<Props, _>(|ee| {
                    assert_eq!(payload.0, ee.payload.0);
                    assert_eq!(err1.kind_id(), ee.kind_id);
                    assert_eq!(err1.msg, ee.msg);
                    assert_eq!(err1.tag(), ee.tag);
                })?
                .with_errorext::<TrivialError, _>(|_| unreachable!("again"))
        });
        assert!(res.is_ok());
    }

    #[test]
    fn test_into_errorext_props() {
        let err = FOO_ERROR.error_with_values(["hi there!".into()]);

        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message: {xyz}");

        let res = err.into_errorext::<Props>();
        match res {
            Ok(ee) => assert_eq!(ee.kind_id, FOO_ERROR.kind_id()),
            Err(_) => unreachable!(),
        };
    }

    #[test]
    fn test_with_errorext_validation() {
        let age_delta: i32 = -10;
        let payload = age_delta
            .validate(
                "age_delta must be nonnegative",
                &Bound::ClosedRange(0, i32::MAX),
            )
            .result()
            .expect_err("validation designed to fail");
        let err = VALIDATION_ERROR.error(payload);
        assert!(err.has_kind(VALIDATION_ERROR.kind_id()));

        let res = swap_result(|| -> ReverseResult<()> {
            err.with_errorext::<TrivialError, _>(|_| unreachable!())?
                .with_errorext::<ValidationError, _>(|ee| {
                    assert_eq!(ee.kind_id, VALIDATION_ERROR.kind_id());
                })?
                .with_errorext::<TrivialError, _>(|_| unreachable!("again"))
        });
        assert!(res.is_ok());
    }
}
