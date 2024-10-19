use super::{BoxPayload, Fmt, JserBoxError, Payload, StdBoxError, WithBacktrace};
use crate::error::static_str::StaticStr;
use crate::error::utils::StringSpec;
use crate::hash::hash_sha256_of_str_arr;
use crate::nodebug::NoDebug;
use crate::string;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::{
    any::type_name,
    backtrace::Backtrace,
    collections::BTreeMap,
    error::Error as StdError,
    fmt::{Debug, Display},
    result,
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
#[derive(Debug, Clone, Copy)]
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
// region:      --- Props

#[derive(Deserialize, Clone, PartialEq, Eq)]
pub struct Props {
    pub(crate) pairs: Vec<(String, String)>,
    pub(crate) protected: bool,
}

impl Props {
    pub fn pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        self.pairs.iter().map(|p| (p.0.as_str(), p.1.as_str()))
    }

    pub fn prop_value(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|&p| p.0 == key)
            .map(|p| p.1.as_str())
    }

    /// Hashes value of fields whose names starts with '!'.
    fn hashed_pairs(&self) -> (Vec<(String, String)>, bool) {
        let mut protected = false;
        let pairs = self
            .pairs
            .iter()
            .map(|(name, value)| {
                let value = if name.starts_with("!") {
                    protected = true;
                    let vhash = hash_sha256_of_str_arr(&[value]);
                    string::base64_encode_trunc_of_u8_arr(&vhash, TRUNC)
                } else {
                    value.to_owned()
                };
                (name.to_owned(), value)
            })
            .collect::<Vec<_>>();
        (pairs, protected)
    }

    pub fn safe_props(&self) -> Self {
        if cfg!(debug_assertions) || self.protected {
            self.clone()
        } else {
            let (pairs, protected) = self.hashed_pairs();
            Self { pairs, protected }
        }
    }
}

impl Debug for Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pairs = if cfg!(debug_assertions) || self.protected {
            &self.pairs
        } else {
            &self.hashed_pairs().0
        };
        f.write_str("Props { pairs: ")?;
        pairs.fmt(f)?;
        f.write_str(", protected: ")?;
        Debug::fmt(&self.protected, f)?;
        f.write_str(" }")
    }
}

impl Serialize for Props {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Props", 2)?;
        if cfg!(debug_assertions) || self.protected {
            state.serialize_field("pairs", &self.pairs)?;
            state.serialize_field("protected", &self.protected)?;
        } else {
            let (pairs, protected) = self.hashed_pairs();
            state.serialize_field("pairs", &pairs)?;
            state.serialize_field("protected", &protected)?;
        };
        state.end()
    }
}

// endregion:   --- Props

//===========================
// region:      --- Error

#[derive(Debug)]
pub struct Error {
    pub(crate) kind_id: &'static KindId,
    pub(super) msg: StaticStr,
    pub(crate) tag: &'static Tag,
    pub(crate) props: Props,
    pub(crate) payload: BoxPayload,
    pub(crate) source: Option<StdBoxError>,
    pub(crate) backtrace: NoDebug<Backtrace>,
    pub(crate) ref_id: Option<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

pub type ReverseResult<T> = std::result::Result<Error, T>;

impl Error {
    pub fn new(
        kind_id: &'static KindId,
        msg: &'static str,
        tag: &'static Tag,
        props: Props,
        payload: impl Payload,
        source: Option<StdBoxError>,
        backtrace: Backtrace,
        ref_id: Option<String>,
    ) -> Self {
        let source = match source {
            Some(e) => Some(StdBoxError::new(e)),
            None => None,
        };
        Self {
            kind_id,
            msg: msg.into(),
            tag,
            props,
            payload: BoxPayload::new(payload),
            source,
            backtrace: NoDebug(backtrace),
            ref_id,
        }
    }

    pub fn with_msg_string(
        kind_id: &'static KindId,
        msg: String,
        tag: &'static Tag,
        props: Props,
        payload: impl Payload,
        source: Option<StdBoxError>,
        backtrace: Backtrace,
        ref_id: Option<String>,
    ) -> Self {
        let source = match source {
            Some(e) => Some(StdBoxError::new(e)),
            None => None,
        };
        Self {
            kind_id,
            msg: msg.into(),
            tag,
            props,
            payload: BoxPayload::new(payload),
            source,
            backtrace: NoDebug(backtrace),
            ref_id,
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

    pub fn props(&self) -> &Props {
        &self.props
    }

    pub fn payload_ref(&self) -> &dyn Payload {
        self.payload.as_ref()
    }

    pub fn try_typed_payload_ref<T: Payload>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }

    pub fn payload_is<T: Payload>(&self) -> bool {
        self.payload.is::<T>()
    }

    /// If the payload is of type `T`, returns `Ok(payload)` , otherwise returns `Err(self)`.
    ///
    /// As this method consumes `self`, if you also need access to other [`Error`] fields then convert
    /// to an [`ErrorExt`] using [`Self::try_into_errorext`] instead.
    pub fn try_typed_payload<T: Payload>(self) -> Result<Box<T>> {
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

    /// Returns `self`'s payload if the payload is of type `T`, panics otherwise.
    ///
    /// Should only be used (instead of [`Self::try_typed_payload`]) when it is known that the payload
    /// is of type `T`, e.g., after calling [`Self::payload_is`].
    ///
    /// As this method consumes `self`, if you also need access to other [`Error`] fields then convert
    /// to an [`ErrorExt`] using [`Self::into_errorext`] instead.
    ///
    /// # Panics
    /// If the payload is not of type `T`.
    pub fn typed_payload<T: Payload>(self) -> Box<T> {
        match self.try_typed_payload::<T>() {
            Ok(pld) => pld,
            _ => panic!("self's payload is not of type {}", type_name::<T>()),
        }
    }

    /// If the payload is of type `T`, returns `Err(f(payload))`; otherwise, returns `Ok(self)`
    /// This unusual signature facilitates chaining of calls of this method for different types.
    ///
    /// As this method consumes `self`, if you also need access to other [`Error`] fields then convert
    /// use [`Error::with_errorext`] instead.
    ///
    /// # Example
    /// ```
    /// use foa::{Error, Result, ReverseResult, error::{Payload, swap_result}};
    /// use std::{any::type_name, fmt::Debug};
    ///
    /// fn process_error<T1: Payload, T2: Payload>(
    ///     err: Error,
    /// ) -> Result<()> {
    ///     swap_result(|| -> ReverseResult<()> {
    ///         err.with_typed_payload::<T1, ()>(|pld| println!("payload type was {}, payload={:?}", type_name::<T1>(), pld))?
    ///             .with_typed_payload::<T2, ()>(|pld| println!("payload type was {}, payload={:?}", type_name::<T2>(), pld))
    ///     })
    /// }
    /// ```
    pub fn with_typed_payload<T: Payload, U>(
        self,
        f: impl FnOnce(Box<T>) -> U,
    ) -> ReverseResult<U> {
        let res = self.try_typed_payload::<T>();
        match res {
            Ok(payload) => Err(f(payload)),
            Err(err) => Ok(err),
        }
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    pub fn ref_id(&self) -> Option<&str> {
        self.ref_id.as_deref()
    }

    /// If the payload is of type `T`, returns `Ok(error_ext)`, where `error_ext` is the
    /// [`ErrorExt`] instance obtained from `self`; otherwise returns `Err(self)`.
    pub fn try_into_errorext<T: Payload>(self) -> Result<ErrorExt<T>> {
        if self.payload.is::<T>() {
            let res = self.payload.downcast::<T>();
            match res {
                Ok(payload) => Ok(ErrorExt {
                    kind_id: self.kind_id,
                    msg: self.msg,
                    tag: self.tag,
                    props: self.props,
                    payload,
                    source: self.source,
                    backtrace: self.backtrace,
                    ref_id: self.ref_id,
                }),
                Err(_) => unreachable!("downcast previously confirmed"),
            }
        } else {
            Err(self)
        }
    }

    /// Returns the [`ErrorExt`] instance obtained from `self` if the payload is of type `T`,
    /// panics otherwise.
    ///
    /// Should only be used (instead of [`Self::try_into_errorext`]) when it is known that the payload
    /// is of type `T`, e.g., after calling [`Self::payload_is`].
    ///
    /// # Panics
    /// If the payload is not of type `T`.
    pub fn into_errorext<T: Payload>(self) -> ErrorExt<T> {
        match self.try_into_errorext::<T>() {
            Ok(ee) => ee,
            _ => panic!("self's payload is not of type {}", type_name::<T>()),
        }
    }

    /// If the payload is of type `T`, returns `Err(f(error_ext))` where `error_ext` is the
    /// [`ErrorExt`] instance obtained from `self`; otherwise, returns `Ok(self)`.
    /// This unusual signature facilitates chaining of calls of this method for different types.
    ///
    /// # Example
    /// ```
    /// use foa::{Error, Result, ReverseResult, error::{Payload, swap_result}};
    /// use std::{any::type_name, fmt::Debug};
    ///
    /// fn process_error<T1: Payload, T2: Payload>(err: Error) -> Result<()> {
    ///     swap_result(|| -> ReverseResult<()> {
    ///         err.with_errorext::<T1, ()>(|ee| {
    ///             println!("payload type was {}, ee={:?}", type_name::<T1>(), ee)
    ///         })?
    ///         .with_errorext::<T2, ()>(|ee| {
    ///             println!("payload type was {}, ee={:?}", type_name::<T2>(), ee)
    ///         })
    ///     })
    /// }
    /// ```
    pub fn with_errorext<T: Payload, U>(
        self,
        f: impl FnOnce(ErrorExt<T>) -> U,
    ) -> ReverseResult<U> {
        let res = self.try_into_errorext::<T>();
        match res {
            Ok(error_ext) => Err(f(error_ext)),
            Err(err) => Ok(err),
        }
    }

    /// Facilitates the chaining of functions that return [`ReverseResult<U>`] by encapsulating the chained call
    /// in argument `f` and using the `?` operator.
    ///
    /// # Example
    /// ```
    /// use foa::{Error, Result, ReverseResult, error::{Payload, swap_result}};
    /// use std::{any::type_name, fmt::Debug};
    ///
    /// fn process_error<T1: Payload, T2: Payload>(err: Error) -> String {
    ///     err.chained_map(
    ///         |err| {
    ///             err.with_typed_payload::<T1, _>(|pld| format!("payload type was {}, payload={:?}", type_name::<T1>(), pld))?
    ///                 .with_typed_payload::<T2, _>(|pld| format!("payload type was {}, payload={:?}", type_name::<T2>(), pld))
    ///         },
    ///         |_err| "payload type was neither `T1` nor `T2`".into(),
    ///     )
    /// }
    /// ```
    pub fn chained_map<U>(
        self,
        f: impl FnOnce(Self) -> ReverseResult<U>,
        fallback: impl FnOnce(Self) -> U,
    ) -> U {
        match f(self) {
            Err(u) => u,
            Ok(err) => fallback(err),
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
            props: self.props.clone(),
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
    props: Props,
    payload: Box<T>,
    source: Option<StdBoxError>,
    backtrace: NoDebug<Backtrace>,
    ref_id: Option<String>,
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

    pub fn props(&self) -> &Props {
        &self.props
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

    pub fn ref_id(&self) -> Option<&str> {
        self.ref_id.as_deref()
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
            props: self.props,
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
        value.try_into_errorext()
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
    pub(super) props: Props,
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

    pub fn props(&self) -> &Props {
        &self.props
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
    pub(super) props: Props,
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

    pub fn props(&self) -> &Props {
        &self.props
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
        error::{swap_result, FullKind, ReverseResult},
        validation::validc::VALIDATION_ERROR,
    };
    use valid::{constraint::Bound, Validate, ValidationError};

    #[derive(Debug, Clone, PartialEq)]
    struct Pld(String);

    static BAR_TAG: Tag = Tag("BAR");

    static BAR_ERROR: FullKind<Pld, 2, false> =
        FullKind::new_with_payload("BAR_ERROR", Some("bar message: {abc}, {!email}"), &BAR_TAG)
            .with_prop_names(["abc", "!email"])
            .with_backtrace(BacktraceSpec::Env);

    fn make_payload_error_pair() -> (Pld, Error) {
        let pld = Pld("bar-payload".into());
        let err =
            BAR_ERROR.error_with_values_and_payload(["hi there", "bar@example.com"], pld.clone());
        (pld, err)
    }

    #[test]
    fn test_extract_payload() {
        let (payload, err) = make_payload_error_pair();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), BAR_ERROR.msg());

        let payload_ext = err.try_typed_payload::<Pld>().unwrap();
        assert_eq!(payload, *payload_ext);
    }

    #[test]
    fn test_with_typed_payload() {
        let (payload, err) = make_payload_error_pair();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), "bar message: {abc}, {!email}");

        let res = swap_result(|| -> ReverseResult<()> {
            err.with_typed_payload::<String, _>(|_| unreachable!())?
                .with_typed_payload::<Pld, _>(|payload_ext| {
                    assert_eq!(payload, *payload_ext);
                })?
                .with_typed_payload::<(), _>(|_| unreachable!("again"))
        });
        assert!(res.is_ok());
    }

    #[test]
    fn test_try_into_errorext_pld() {
        let (_, err) = make_payload_error_pair();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), "bar message: {abc}, {!email}");

        let res = err.try_into_errorext::<Pld>();
        match res {
            Ok(ee) => assert_eq!(ee.kind_id, BAR_ERROR.kind_id()),
            Err(_) => unreachable!(),
        };
    }

    #[test]
    fn test_into_errorext_pld() {
        let (payload, err) = make_payload_error_pair();
        let (_, err1) = make_payload_error_pair();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), "bar message: {abc}, {!email}");

        if err.payload_is::<String>() {
            unreachable!()
        } else if err.payload_is::<Pld>() {
            let ee = err.into_errorext();
            assert_eq!(payload, *ee.payload);
            assert_eq!(err1.kind_id(), ee.kind_id);
            assert_eq!(err1.msg, ee.msg);
            assert_eq!(err1.tag(), ee.tag);
        } else if err.payload_is::<()>() {
            unreachable!("again")
        };
    }

    #[test]
    fn test_with_errorext_pld() {
        let (payload, err) = make_payload_error_pair();
        let (_, err1) = make_payload_error_pair();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), "bar message: {abc}, {!email}");

        let res = swap_result(|| -> ReverseResult<()> {
            err.with_errorext::<String, _>(|_| unreachable!())?
                .with_errorext::<Pld, _>(|ee| {
                    assert_eq!(payload, *ee.payload);
                    assert_eq!(err1.kind_id(), ee.kind_id);
                    assert_eq!(err1.msg, ee.msg);
                    assert_eq!(err1.tag(), ee.tag);
                })?
                .with_errorext::<(), _>(|_| unreachable!("again"))
        });
        assert!(res.is_ok());
    }

    #[test]
    fn test_chained_map_pld() {
        let (payload, err) = make_payload_error_pair();
        let (_, err1) = make_payload_error_pair();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), "bar message: {abc}, {!email}");

        err.chained_map(
            |err| {
                err.with_errorext::<String, _>(|_| unreachable!())?
                    .with_errorext::<Pld, _>(|ee| {
                        assert_eq!(payload, *ee.payload);
                        assert_eq!(err1.kind_id(), ee.kind_id);
                        assert_eq!(err1.msg, ee.msg);
                        assert_eq!(err1.tag(), ee.tag);
                    })?
                    .with_errorext::<(), _>(|_| unreachable!("again"))
            },
            |_err| unreachable!("fallback shouldn't execute"),
        )
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
        let err = VALIDATION_ERROR.error_with_payload(payload);
        assert!(err.has_kind(VALIDATION_ERROR.kind_id()));

        let res = swap_result(|| -> ReverseResult<()> {
            err.with_errorext::<String, _>(|_| unreachable!())?
                .with_errorext::<ValidationError, _>(|ee| {
                    assert_eq!(ee.kind_id, VALIDATION_ERROR.kind_id());
                })?
                .with_errorext::<(), _>(|_| unreachable!("again"))
        });
        assert!(res.is_ok());
    }
}
