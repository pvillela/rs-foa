use super::{
    ref_id_u32_hex_lower, BacktraceSpec, Error, KindId, KindTypeInfo, NullError, Payload, Props,
    SendSyncStaticError, StdBoxError, Tag,
};
use std::backtrace::Backtrace;
use std::fmt::Debug;
use std::marker::PhantomData;

//===========================
// region:      --- Kind types and aliases

#[derive(Debug)]
pub struct FullKind<PLD: Payload, const ARITY: usize, SRC: SendSyncStaticError = NullError> {
    pub(super) kind_id: KindId,
    pub(super) msg: Option<&'static str>,
    pub(super) tag: &'static Tag,
    pub(super) prop_names: [&'static str; ARITY],
    pub(super) backtrace_spec: BacktraceSpec,
    has_ref_id: bool,
    _pld: PhantomData<PLD>,
    _src: PhantomData<SRC>,
}

pub type BasicKind<SRC = NullError> = FullKind<(), 0, SRC>;

pub type PropsKind<const ARITY: usize, SRC = NullError> = FullKind<(), ARITY, SRC>;

pub type PayloadKind<PLD, SRC = NullError> = FullKind<PLD, 0, SRC>;

// endregion:   --- Kind types and aliases

//===========================
// region:      --- Kind constructors

impl<SRC: SendSyncStaticError> BasicKind<SRC> {
    pub const fn new(name: &'static str, msg: Option<&'static str>, tag: &'static Tag) -> Self {
        Self {
            kind_id: KindId(name),
            msg,
            tag,
            prop_names: [],
            backtrace_spec: BacktraceSpec::No,
            has_ref_id: false,
            _pld: PhantomData,
            _src: PhantomData,
        }
    }
}

impl<PLD: Payload, SRC: SendSyncStaticError> PayloadKind<PLD, SRC> {
    pub const fn new_with_payload(
        name: &'static str,
        msg: Option<&'static str>,
        tag: &'static Tag,
    ) -> Self {
        Self {
            kind_id: KindId(name),
            msg,
            tag,
            prop_names: [],
            backtrace_spec: BacktraceSpec::No,
            has_ref_id: false,
            _pld: PhantomData,
            _src: PhantomData,
        }
    }
}

impl<PLD: Payload, SRC: SendSyncStaticError> FullKind<PLD, 0, SRC> {
    pub const fn with_prop_names<const ARITY: usize>(
        self,
        prop_names: [&'static str; ARITY],
    ) -> FullKind<PLD, ARITY, SRC> {
        FullKind {
            kind_id: self.kind_id,
            msg: self.msg,
            tag: self.tag,
            prop_names,
            backtrace_spec: self.backtrace_spec,
            has_ref_id: self.has_ref_id,
            _pld: PhantomData,
            _src: PhantomData,
        }
    }
}

impl<PLD: Payload, const ARITY: usize, SRC: SendSyncStaticError> FullKind<PLD, ARITY, SRC> {
    pub const fn with_backtrace(self, backtrace_spec: BacktraceSpec) -> Self {
        Self {
            backtrace_spec,
            ..self
        }
    }

    pub const fn with_ref_id(self) -> Self {
        Self {
            has_ref_id: true,
            ..self
        }
    }

    pub const fn with_payload<T: Payload>(self) -> FullKind<T, ARITY, SRC> {
        FullKind {
            kind_id: self.kind_id,
            msg: self.msg,
            tag: self.tag,
            prop_names: self.prop_names,
            backtrace_spec: self.backtrace_spec,
            has_ref_id: self.has_ref_id,
            _pld: PhantomData,
            _src: PhantomData,
        }
    }
}

impl<PLD: Payload, const ARITY: usize, SRC: SendSyncStaticError> FullKind<PLD, ARITY, SRC> {
    pub const fn with_src<T: SendSyncStaticError>(self) -> FullKind<PLD, ARITY, T> {
        FullKind {
            kind_id: self.kind_id,
            msg: self.msg,
            tag: self.tag,
            prop_names: self.prop_names,
            backtrace_spec: self.backtrace_spec,
            has_ref_id: self.has_ref_id,
            _pld: PhantomData,
            _src: PhantomData,
        }
    }
}

// endregion:   --- Kind constructors

//===========================
// region:      --- Accessors

impl<PLD: Payload, const ARITY: usize, SRC: SendSyncStaticError> FullKind<PLD, ARITY, SRC> {
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

    pub const fn prop_names(&self) -> &[&'static str; ARITY] {
        &self.prop_names
    }

    pub const fn backtrace_spec(&self) -> BacktraceSpec {
        self.backtrace_spec
    }

    pub const fn has_ref_id(&self) -> bool {
        self.has_ref_id
    }
}

// endregion:   --- Accessors

//===========================
// region:      --- Error constructors

impl<PLD: Payload, const ARITY: usize, SRC: SendSyncStaticError> FullKind<PLD, ARITY, SRC> {
    fn error_priv(
        &'static self,
        values: [&str; ARITY],
        payload: PLD,
        source: Option<StdBoxError>,
    ) -> Error {
        let pairs = self
            .prop_names
            .into_iter()
            .zip(values)
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect::<Vec<_>>();
        let props = Props {
            pairs,
            protected: false,
        };
        let msg = match self.msg {
            Some(msg) => msg,
            None => self.kind_id.0,
        };
        let backtrace = match self.backtrace_spec {
            BacktraceSpec::Yes => Backtrace::force_capture(),
            BacktraceSpec::No => Backtrace::disabled(),
            BacktraceSpec::Env => Backtrace::capture(),
        };

        let ref_id = if self.has_ref_id {
            Some(ref_id_u32_hex_lower())
        } else {
            None
        };

        Error::new(
            self.kind_id(),
            msg.into(),
            self.tag,
            props,
            payload,
            source,
            backtrace,
            ref_id,
        )
    }
}

impl BasicKind {
    pub fn error(&'static self) -> Error {
        self.error_priv([], (), None)
    }
}

impl<SRC: SendSyncStaticError> BasicKind<SRC> {
    pub fn error_with_src(&'static self, source: SRC) -> Error {
        self.error_priv([], (), Some(StdBoxError::new(source)))
    }
}

impl<const ARITY: usize> PropsKind<ARITY> {
    pub fn error_with_values(&'static self, values: [&str; ARITY]) -> Error {
        self.error_priv(values, (), None)
    }
}

impl<const ARITY: usize, SRC: SendSyncStaticError> PropsKind<ARITY, SRC> {
    pub fn error_with_values_src(&'static self, values: [&str; ARITY], source: SRC) -> Error {
        self.error_priv(values, (), Some(StdBoxError::new(source)))
    }
}

impl<PLD: Payload> FullKind<PLD, 0> {
    pub fn error_with_payload(&'static self, payload: PLD) -> Error {
        self.error_priv([], payload, None)
    }
}

impl<PLD: Payload, SRC: SendSyncStaticError> FullKind<PLD, 0, SRC> {
    pub fn error_with_payload_src(&'static self, payload: PLD, source: SRC) -> Error {
        self.error_priv([], payload, Some(StdBoxError::new(source)))
    }
}

impl<PLD: Payload, const ARITY: usize> FullKind<PLD, ARITY> {
    pub fn error_with_values_payload(&'static self, values: [&str; ARITY], payload: PLD) -> Error {
        self.error_priv(values, payload, None)
    }
}

impl<PLD: Payload, const ARITY: usize, SRC: SendSyncStaticError> FullKind<PLD, ARITY, SRC> {
    pub fn error_with_values_payload_src(
        &'static self,
        values: [&str; ARITY],
        payload: PLD,
        source: SRC,
    ) -> Error {
        self.error_priv(values, payload, Some(StdBoxError::new(source)))
    }
}

// endregion:   --- Error constructors

//===========================
// region:      --- impl KindTypeInfo

impl<PLD: Payload, const ARITY: usize, SRC: SendSyncStaticError> KindTypeInfo
    for FullKind<PLD, ARITY, SRC>
{
    type Pld = PLD;
    type Src = SRC;
}

// endregion:   --- impl KindTypeInfo

#[cfg(test)]
mod test_props_kind {
    use super::{BasicKind, PropsKind};
    use crate::error::{BacktraceSpec, Tag};

    static FOO_TAG: Tag = Tag("FOO");

    static FOO_ERROR: PropsKind<1> =
        BasicKind::new("FOO_ERROR", Some("foo message: {xyz}"), &FOO_TAG)
            .with_prop_names(["xyz"])
            .with_backtrace(BacktraceSpec::Env);

    #[test]
    fn test() {
        let err = FOO_ERROR.error_with_values(["hi there!"]);
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message: {xyz}");
    }
}

#[cfg(test)]
mod test_basic_kind {
    use super::BasicKind;
    use crate::error::{BacktraceSpec, Tag};

    static FOO_TAG: Tag = Tag("FOO");

    static FOO_ERROR: BasicKind =
        BasicKind::new("FOO_ERROR", None, &FOO_TAG).with_backtrace(BacktraceSpec::Env);

    #[test]
    fn test() {
        let err = FOO_ERROR.error();
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "FOO_ERROR");
    }
}

#[cfg(test)]
mod test_payload_kind {
    use super::PayloadKind;
    use crate::error::{BacktraceSpec, Tag};

    static FOO_TAG: Tag = Tag("FOO");

    static FOO_ERROR: PayloadKind<String> =
        PayloadKind::new_with_payload("FOO_ERROR", None, &FOO_TAG)
            .with_backtrace(BacktraceSpec::Env);

    #[test]
    fn test() {
        let err = FOO_ERROR.error_with_payload("dummy payload".to_owned());
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "FOO_ERROR".to_string());
    }
}
