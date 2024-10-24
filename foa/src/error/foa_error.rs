use super::{
    BoxPayload, Fmt, KindId, KindTypeInfo, NullError, Payload, Props, SendSyncStaticError,
    SerError, StaticStr, StdBoxError, StringSpec, Tag, WithBacktrace,
};
use crate::nodebug::NoDebug;
use serde::Serialize;
use std::{
    any::{type_name, TypeId},
    backtrace::Backtrace,
    collections::BTreeMap,
    error::Error as StdError,
    fmt::{Debug, Display},
};

pub const TRUNC: usize = 8;

//===========================
// region:      --- Aliases

// pub type StdBoxError = Box<dyn StdError + Send + Sync + 'static>;

pub type Result<T, PLD = BoxPayload, SRC = StdBoxError> = std::result::Result<T, Error<PLD, SRC>>;

pub type ReverseResult<T, PLD = BoxPayload, SRC = StdBoxError> =
    std::result::Result<Error<PLD, SRC>, T>;

//===========================
// region:      --- Error type and constructors

#[derive(Debug)]
pub struct Error<PLD = BoxPayload, SRC = StdBoxError> {
    pub(crate) kind_id: &'static KindId,
    pub(super) msg: StaticStr,
    pub(crate) tag: &'static Tag,
    pub(crate) props: Props,
    pub(crate) payload: PLD,
    pub(crate) src: Option<SRC>,
    pub(crate) backtrace: NoDebug<Backtrace>,
    pub(crate) ref_id: Option<String>,
}

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
        Self {
            kind_id,
            msg: msg.into(),
            tag,
            props,
            payload: BoxPayload::new(payload),
            src: source,
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
        Self {
            kind_id,
            msg: msg.into(),
            tag,
            props,
            payload: BoxPayload::new(payload),
            src: source,
            backtrace: NoDebug(backtrace),
            ref_id,
        }
    }
}

// endregion:   --- Error type and constructors

//===========================
// region:      --- Error accessor methods

impl<PLD: Payload, SRC: SendSyncStaticError> Error<PLD, SRC> {
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

    pub fn payload(&self) -> &PLD {
        &self.payload
    }

    pub fn src(&self) -> Option<&SRC> {
        self.src.as_ref()
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

    pub fn to_sererror_without_pld_or_src<const N: usize>(
        &self,
        str_specs: [StringSpec; N],
    ) -> SerError<(), NullError> {
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
            payload: None,
            src: None,
            other,
        }
    }

    pub fn into_sererror_with_pld<const N: usize>(
        self,
        str_specs: [StringSpec; N],
    ) -> SerError<PLD, NullError>
    where
        PLD: Serialize,
    {
        let fmt = Fmt(&self);
        let other = str_specs
            .into_iter()
            .map(|spec| fmt.speced_string_tuple(&spec))
            .collect::<BTreeMap<&'static str, String>>();
        SerError {
            kind_id: self.kind_id,
            msg: self.msg.clone(),
            tag: self.tag,
            props: self.props.clone(),
            payload: Some(self.payload),
            src: None,
            other,
        }
    }

    pub fn into_sererror_with_src<const N: usize>(
        self,
        str_specs: [StringSpec; N],
    ) -> SerError<(), SRC>
    where
        SRC: Serialize,
    {
        let fmt = Fmt(&self);
        let other = str_specs
            .into_iter()
            .map(|spec| fmt.speced_string_tuple(&spec))
            .collect::<BTreeMap<&'static str, String>>();
        SerError {
            kind_id: self.kind_id,
            msg: self.msg.clone(),
            tag: self.tag,
            props: self.props.clone(),
            payload: None,
            src: self.src,
            other,
        }
    }

    pub fn into_sererror_with_pld_and_src<const N: usize>(
        self,
        str_specs: [StringSpec; N],
    ) -> SerError<PLD, SRC>
    where
        PLD: Serialize,
        SRC: Serialize,
    {
        let fmt = Fmt(&self);
        let other = str_specs
            .into_iter()
            .map(|spec| fmt.speced_string_tuple(&spec))
            .collect::<BTreeMap<&'static str, String>>();
        SerError {
            kind_id: self.kind_id,
            msg: self.msg.clone(),
            tag: self.tag,
            props: self.props.clone(),
            payload: Some(self.payload),
            src: self.src,
            other,
        }
    }
}

impl<SRC: SendSyncStaticError> Error<BoxPayload, SRC> {
    pub fn payload_ref(&self) -> &dyn Payload {
        self.payload.as_ref()
    }
}

// endregion:   --- Error accessor methods

//===========================
// region:      --- Error downcast methods

impl<SRC: SendSyncStaticError> Error<BoxPayload, SRC> {
    pub fn payload_is<T: Payload>(&self) -> bool {
        self.payload.is::<T>()
    }

    pub fn downcast_payload_ref<T: Payload>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }

    /// If the payload is of type `T`, returns `Ok(error_ext)`, where `error_ext` is
    /// `self` with the `Box dyn` payload replaced by a `Box<T>`; otherwise returns `Err(self)`.
    pub fn downcast_payload<T: Payload>(self) -> Result<Error<Box<T>, SRC>, BoxPayload, SRC> {
        if self.payload.is::<T>() {
            let res = self.payload.downcast::<T>();
            match res {
                Ok(payload) => Ok(Error {
                    kind_id: self.kind_id,
                    msg: self.msg,
                    tag: self.tag,
                    props: self.props,
                    payload,
                    src: self.src,
                    backtrace: self.backtrace,
                    ref_id: self.ref_id,
                }),
                Err(_) => unreachable!("downcast previously confirmed"),
            }
        } else {
            Err(self)
        }
    }

    /// If the payload is of type `T`, returns `error_ext`, where `error_ext` is
    /// `self` with the `Box dyn` payload replaced by a `Box<T>`; panics otherwise.
    ///
    /// Should only be used (instead of [`Self::try_into_errorext`]) when it is known that the payload
    /// is of type `T`, e.g., after calling [`Self::payload_is`].
    ///
    /// # Panics
    /// If the payload is not of type `T`.
    pub fn force_downcast_payload<T: Payload>(self) -> Error<Box<T>, SRC> {
        match self.downcast_payload::<T>() {
            Ok(ee) => ee,
            _ => panic!("self's payload is not of type {}", type_name::<T>()),
        }
    }

    /// If the payload is of type `T`, returns `Err(f(error_ext))` where `error_ext` is
    /// `self` with the `Box dyn` payload replaced by a `Box<T>`; otherwise, returns `Ok(self)`.
    /// This unusual signature facilitates chaining of calls of this method for different types.
    ///
    /// # Example
    /// ```
    /// use foa::{Error, Result, ReverseResult, error::{Payload, swap_result}};
    /// use std::{any::type_name, fmt::Debug};
    ///
    /// fn process_error<T1: Payload, T2: Payload>(err: Error) -> Result<()> {
    ///     swap_result(|| -> ReverseResult<()> {
    ///         err.with_downcast_payload::<T1, ()>(|ee| {
    ///             println!("payload type was {}, ee={:?}", type_name::<T1>(), ee)
    ///         })?
    ///         .with_downcast_payload::<T2, ()>(|ee| {
    ///             println!("payload type was {}, ee={:?}", type_name::<T2>(), ee)
    ///         })
    ///     })
    /// }
    /// ```
    pub fn with_downcast_payload<T: Payload, U>(
        self,
        f: impl FnOnce(Error<Box<T>, SRC>) -> U,
    ) -> ReverseResult<U, BoxPayload, SRC> {
        let res = self.downcast_payload::<T>();
        match res {
            Ok(error_ext) => Err(f(error_ext)),
            Err(err) => Ok(err),
        }
    }
}

impl<PLD: Payload> Error<PLD, StdBoxError> {
    pub fn source_is<T: SendSyncStaticError>(&self) -> bool {
        match &self.src {
            Some(y) => y.0.is::<T>(),
            None => false,
        }
    }

    /// Returns some reference to the `source` field if it is of type `S`, or
    /// `None` if it isn't.
    pub fn downcast_source_ref<S: StdError + Send + Sync + 'static>(&self) -> Option<&S> {
        match &self.src {
            Some(y) => y.0.downcast_ref(),
            None => None,
        }
    }

    /// If the source is of type `T`, returns `Ok(error_ext)`, where `error_ext` is
    /// `self` with the `Box dyn` source replaced by a `Box<T>`; otherwise returns `Err(self)`.
    pub fn downcast_source<T: SendSyncStaticError>(self) -> Result<Error<PLD, Box<T>>, PLD> {
        if self.source_is::<T>() {
            let res = self.src.unwrap().0.downcast::<T>();
            match res {
                Ok(source) => Ok(Error {
                    kind_id: self.kind_id,
                    msg: self.msg,
                    tag: self.tag,
                    props: self.props,
                    payload: self.payload,
                    src: Some(source),
                    backtrace: self.backtrace,
                    ref_id: self.ref_id,
                }),
                Err(_) => unreachable!("downcast previously confirmed"),
            }
        } else {
            Err(self)
        }
    }

    /// If the source is of type `T`, returns `error_ext`, where `error_ext` is
    /// `self` with the `Box dyn` source replaced by a `Box<T>`; panics otherwise.
    ///
    /// Should only be used (instead of [`Self::try_into_errorext`]) when it is known that the source
    /// is of type `T`, e.g., after calling [`Self::source_is`].
    ///
    /// # Panics
    /// If the source is not of type `T`.
    pub fn force_downcast_source<T: SendSyncStaticError>(self) -> Error<PLD, Box<T>> {
        match self.downcast_source::<T>() {
            Ok(ee) => ee,
            _ => panic!("self's source is not of type {}", type_name::<T>()),
        }
    }

    /// If the source is of type `T`, returns `Err(f(error_ext))` where `error_ext` is
    /// `self` with the `Box dyn` source replaced by a `Box<T>`; otherwise, returns `Ok(self)`.
    /// This unusual signature facilitates chaining of calls of this method for different types.
    ///
    /// # Example
    /// ```
    /// use foa::{Error, Result, ReverseResult, error::{SendSyncStaticError, swap_result}};
    /// use std::{any::type_name, fmt::Debug};
    ///
    /// fn process_error<T1: SendSyncStaticError, T2: SendSyncStaticError>(err: Error) -> Result<()> {
    ///     swap_result(|| -> ReverseResult<()> {
    ///         err.with_downcast_source::<T1, ()>(|ee| {
    ///             println!("source type was {}, ee={:?}", type_name::<T1>(), ee)
    ///         })?
    ///         .with_downcast_source::<T2, ()>(|ee| {
    ///             println!("source type was {}, ee={:?}", type_name::<T2>(), ee)
    ///         })
    ///     })
    /// }
    /// ```
    pub fn with_downcast_source<T: SendSyncStaticError, U>(
        self,
        f: impl FnOnce(Error<PLD, Box<T>>) -> U,
    ) -> ReverseResult<U, PLD> {
        let res = self.downcast_source::<T>();
        match res {
            Ok(error_ext) => Err(f(error_ext)),
            Err(err) => Ok(err),
        }
    }
}

impl Error {
    pub fn downcast_payload_source<PLD: Payload, SRC: SendSyncStaticError>(
        self,
    ) -> Result<Error<Box<PLD>, Box<SRC>>> {
        let res1 = self.downcast_payload::<PLD>()?;
        let res2 = res1.downcast_source::<SRC>();
        match res2 {
            Ok(de) => Ok(de),
            Err(err) => Err(Error {
                kind_id: err.kind_id,
                tag: err.tag,
                msg: err.msg,
                props: err.props,
                payload: BoxPayload::new(*err.payload),
                src: err.src,
                backtrace: err.backtrace,
                ref_id: err.ref_id,
            }),
        }
    }

    pub fn downcast_for_kind<K: KindTypeInfo>(
        self,
        _kind: &K,
    ) -> Result<Error<Box<K::Pld>, Box<K::Src>>> {
        if TypeId::of::<K::Src>() == TypeId::of::<NullError>() {
            let derr = self.downcast_payload::<K::Pld>()?;
            Ok(Error {
                kind_id: derr.kind_id,
                tag: derr.tag,
                msg: derr.msg,
                props: derr.props,
                payload: derr.payload,
                src: None,
                backtrace: derr.backtrace,
                ref_id: derr.ref_id,
            })
        } else {
            self.downcast_payload_source::<K::Pld, K::Src>()
        }
    }
}

impl<PLD: Payload, SRC: SendSyncStaticError> Error<PLD, SRC> {
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
    ///             err.with_downcast_payload::<T1, _>(|pld| format!("payload type was {}, payload={:?}", type_name::<T1>(), pld))?
    ///                 .with_downcast_payload::<T2, _>(|pld| format!("payload type was {}, payload={:?}", type_name::<T2>(), pld))
    ///         },
    ///         |_err| "payload type was neither `T1` nor `T2`".into(),
    ///     )
    /// }
    /// ```
    pub fn chained_map<U>(
        self,
        f: impl FnOnce(Self) -> ReverseResult<U, PLD, SRC>,
        fallback: impl FnOnce(Self) -> U,
    ) -> U {
        match f(self) {
            Err(u) => u,
            Ok(err) => fallback(err),
        }
    }
}

// endregion:   --- Error downcast methods

//===========================
// region:      --- Error trait impls

impl<PLD: Payload, SRC: SendSyncStaticError> Display for Error<PLD, SRC> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl<PLD: Payload, SRC: SendSyncStaticError> StdError for Error<PLD, SRC> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.src {
            Some(e) => Some(e),
            None => None,
        }
    }
}

impl<PLD: Payload, SRC: SendSyncStaticError> WithBacktrace for Error<PLD, SRC> {
    fn backtrace(&self) -> &Backtrace {
        Self::backtrace(&self)
    }
}

// endregion:   --- Error trait impls

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        error::{
            recursive_msg, swap_result, BacktraceSpec, ErrSrcNotTyped, FullKind, ReverseResult,
            TrivialError,
        },
        validation::validc::VALIDATION_ERROR,
    };
    use std::any::Any;
    use valid::{constraint::Bound, Validate, ValidationError};

    #[derive(Debug, Clone, PartialEq)]
    struct Pld(String);

    static BAR_TAG: Tag = Tag("BAR");

    static BAR_ERROR: FullKind<Pld, 2, ErrSrcNotTyped> =
        FullKind::new_with_payload("BAR_ERROR", Some("bar message: {abc}, {!email}"), &BAR_TAG)
            .with_prop_names(["abc", "!email"])
            .with_backtrace(BacktraceSpec::Env);

    fn make_payload_source_error_tuple() -> (Pld, TrivialError, Error) {
        let pld = Pld("bar-payload".into());
        let src = TrivialError("dummy");
        let err = BAR_ERROR.error_with_values_payload(
            ["hi there", "bar@example.com"],
            pld.clone(),
            src.clone(),
        );
        (pld, src, err)
    }

    #[test]
    fn test_downcast_payload_ref() {
        let (payload, _, err) = make_payload_source_error_tuple();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), BAR_ERROR.msg());

        let payload_ext = err.downcast_payload_ref::<Pld>().unwrap();
        assert_eq!(&payload, payload_ext);
    }

    #[test]
    fn test_downcast_payload() {
        let (payload, _, err) = make_payload_source_error_tuple();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), BAR_ERROR.msg());

        let err_ext = err.downcast_payload::<Pld>().unwrap();
        assert_eq!(&payload, err_ext.payload().as_ref());
    }

    #[test]
    fn test_with_downcast_payload() {
        let (payload, _, err) = make_payload_source_error_tuple();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), "bar message: {abc}, {!email}");

        let res = swap_result(|| -> ReverseResult<()> {
            err.with_downcast_payload::<String, _>(|_| unreachable!())?
                .with_downcast_payload::<Pld, _>(|err_ext| {
                    assert_eq!(&payload, err_ext.payload().as_ref());
                })?
                .with_downcast_payload::<(), _>(|_| unreachable!("again"))
        });
        assert!(res.is_ok());
    }

    #[test]
    fn test_downcast_source_ref() {
        let (_, src, err) = make_payload_source_error_tuple();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), BAR_ERROR.msg());

        println!("err={:?}", err);
        println!("recursive_msg={}", recursive_msg(&err));
        println!("src.type_id()={:?}", Any::type_id(&src));

        let source_ext = err.downcast_source_ref::<TrivialError>().unwrap();
        assert_eq!(&src, source_ext);
    }

    #[test]
    fn test_try_into_errorext_pld_and_errorext_downcast_source_ref() {
        let (_, src, err) = make_payload_source_error_tuple();

        println!("src.type_id()={:?}", Any::type_id(&src));

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), "bar message: {abc}, {!email}");

        let res = err.downcast_payload::<Pld>();
        match res {
            Ok(ee) => {
                println!("ee={:?}", ee);
                println!("recursive_msg={}", recursive_msg(&ee));

                assert_eq!(ee.kind_id, BAR_ERROR.kind_id());

                let source_ext = ee.downcast_source_ref::<TrivialError>().unwrap();
                assert_eq!(&src, source_ext);
            }
            Err(_) => unreachable!(),
        };
    }

    #[test]
    fn test_into_errorext_pld() {
        let (payload, _, err) = make_payload_source_error_tuple();
        let (_, _, err1) = make_payload_source_error_tuple();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), "bar message: {abc}, {!email}");

        if err.payload_is::<String>() {
            unreachable!()
        } else if err.payload_is::<Pld>() {
            let ee = err.force_downcast_payload();
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
        let (payload, _, err) = make_payload_source_error_tuple();
        let (_, _, err1) = make_payload_source_error_tuple();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), "bar message: {abc}, {!email}");

        let res = swap_result(|| -> ReverseResult<()> {
            err.with_downcast_payload::<String, _>(|_| unreachable!())?
                .with_downcast_payload::<Pld, _>(|ee| {
                    assert_eq!(payload, *ee.payload);
                    assert_eq!(err1.kind_id(), ee.kind_id);
                    assert_eq!(err1.msg, ee.msg);
                    assert_eq!(err1.tag(), ee.tag);
                })?
                .with_downcast_payload::<(), _>(|_| unreachable!("again"))
        });
        assert!(res.is_ok());
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
            err.with_downcast_payload::<String, _>(|_| unreachable!())?
                .with_downcast_payload::<ValidationError, _>(|ee| {
                    assert_eq!(ee.kind_id, VALIDATION_ERROR.kind_id());
                })?
                .with_downcast_payload::<(), _>(|_| unreachable!("again"))
        });
        assert!(res.is_ok());
    }

    #[test]
    fn test_chained_map_pld() {
        let (payload, _, err) = make_payload_source_error_tuple();
        let (_, _, err1) = make_payload_source_error_tuple();

        assert!(err.has_kind(BAR_ERROR.kind_id()));
        assert_eq!(err.to_string(), "bar message: {abc}, {!email}");

        err.chained_map(
            |err| {
                err.with_downcast_payload::<String, _>(|_| unreachable!())?
                    .with_downcast_payload::<Pld, _>(|ee| {
                        assert_eq!(payload, *ee.payload);
                        assert_eq!(err1.kind_id(), ee.kind_id);
                        assert_eq!(err1.msg, ee.msg);
                        assert_eq!(err1.tag(), ee.tag);
                    })?
                    .with_downcast_payload::<(), _>(|_| unreachable!("again"))
            },
            |_err| unreachable!("fallback shouldn't execute"),
        )
    }
}
