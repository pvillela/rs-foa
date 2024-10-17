use super::{BacktraceSpec, Error, KindId, Payload, StdBoxError, Tag};
use crate::error::foa_error::Props;
use std::backtrace::Backtrace;
use std::marker::PhantomData;
use std::{error::Error as StdError, fmt::Debug};

//===========================
// region:      --- FullKind

#[derive(Debug)]
pub struct FullKind<PLD: Payload, const ARITY: usize, const HASCAUSE: bool> {
    pub(super) kind_id: KindId,
    pub(super) msg: Option<&'static str>,
    pub(super) prop_names: [&'static str; ARITY],
    pub(super) backtrace_spec: BacktraceSpec,
    pub(super) tag: &'static Tag,
    _pld: PhantomData<PLD>,
}

impl<PLD: Payload, const ARITY: usize, const HASCAUSE: bool> FullKind<PLD, ARITY, HASCAUSE> {
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

    pub const fn new_with_prop_names(
        name: &'static str,
        msg: Option<&'static str>,
        prop_names: [&'static str; ARITY],
        backtrace_spec: BacktraceSpec,
        tag: &'static Tag,
    ) -> Self {
        Self {
            kind_id: KindId(name),
            msg,
            prop_names,
            backtrace_spec,
            tag,
            _pld: PhantomData,
        }
    }

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

        Error::new(
            self.kind_id(),
            msg.into(),
            self.tag,
            props,
            payload,
            source,
            backtrace,
        )
    }
}

impl<PLD: Payload, const ARITY: usize> FullKind<PLD, ARITY, false> {
    pub fn error_with_values_and_payload(
        &'static self,
        values: [&str; ARITY],
        payload: PLD,
    ) -> Error {
        self.error_priv(values, payload, None)
    }
}

impl<PLD: Payload, const ARITY: usize> FullKind<PLD, ARITY, true> {
    pub fn error_with_values_and_payload(
        &'static self,
        values: [&str; ARITY],
        payload: PLD,
        cause: impl StdError + Send + Sync + 'static,
    ) -> Error {
        self.error_priv(values, payload, Some(StdBoxError::new(cause)))
    }
}

// endregion:   --- FullKind

//===========================
// region:      --- PropsKind

pub type PropsKind<const ARITY: usize, const HASCAUSE: bool> = FullKind<(), ARITY, HASCAUSE>;

impl<const ARITY: usize> PropsKind<ARITY, false> {
    pub fn error_with_values(&'static self, values: [&str; ARITY]) -> Error {
        self.error_priv(values, (), None)
    }
}

impl<const ARITY: usize> PropsKind<ARITY, true> {
    pub fn error_with_values(
        &'static self,
        values: [&str; ARITY],
        cause: impl StdError + Send + Sync + 'static,
    ) -> Error {
        self.error_priv(values, (), Some(StdBoxError::new(cause)))
    }
}

// endregion:   --- PropsKind

//===========================
// region:      --- BasicKind

pub type BasicKind<const HASCAUSE: bool> = PropsKind<0, HASCAUSE>;

impl<const HASCAUSE: bool> BasicKind<HASCAUSE> {
    pub const fn new_basic_kind(
        name: &'static str,
        msg: Option<&'static str>,
        backtrace_spec: BacktraceSpec,
        tag: &'static Tag,
    ) -> Self {
        Self {
            kind_id: KindId(name),
            msg,
            prop_names: [],
            backtrace_spec,
            tag,
            _pld: PhantomData,
        }
    }
}

impl BasicKind<false> {
    pub fn error(&'static self) -> Error {
        self.error_priv([], (), None)
    }
}

impl BasicKind<true> {
    pub fn error(&'static self, cause: impl StdError + Send + Sync + 'static) -> Error {
        self.error_priv([], (), Some(StdBoxError::new(cause)))
    }
}

// endregion:   --- BasicKind

//===========================
// region:      --- PayloadKind

pub type PayloadKind<PLD, const HASCAUSE: bool> = FullKind<PLD, 0, HASCAUSE>;

impl<PLD: Payload, const HASCAUSE: bool> PayloadKind<PLD, HASCAUSE> {
    pub const fn new_payloadkind(
        name: &'static str,
        msg: Option<&'static str>,
        backtrace_spec: BacktraceSpec,
        tag: &'static Tag,
    ) -> Self {
        Self::new_with_prop_names(name, msg, [], backtrace_spec, tag)
    }
}

impl<PLD: Payload> PayloadKind<PLD, false> {
    pub fn error_with_payload(&'static self, payload: PLD) -> Error {
        self.error_with_values_and_payload([], payload)
    }
}

impl<PLD: Payload> PayloadKind<PLD, true> {
    pub fn error_with_payload(
        &'static self,
        payload: PLD,
        source: impl StdError + Send + Sync + 'static,
    ) -> Error {
        self.error_with_values_and_payload([], payload, StdBoxError::new(source))
    }
}

// endregion:   --- PayloadKind

#[cfg(test)]
mod test_props_kind {
    use super::PropsKind;
    use crate::error::{BacktraceSpec, Tag};

    static FOO_TAG: Tag = Tag("FOO");

    static FOO_ERROR: PropsKind<1, false> = PropsKind::new_with_prop_names(
        "FOO_ERROR",
        Some("foo message: {xyz}"),
        ["xyz"],
        BacktraceSpec::Env,
        &FOO_TAG,
    );

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

    static FOO_ERROR: BasicKind<false> =
        BasicKind::new_basic_kind("FOO_ERROR", None, BacktraceSpec::Env, &FOO_TAG);

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

    static FOO_ERROR: PayloadKind<String, false> =
        PayloadKind::new_payloadkind("FOO_ERROR", None, BacktraceSpec::Env, &FOO_TAG);

    #[test]
    fn test() {
        let err = FOO_ERROR.error_with_payload("dummy payload".to_owned());
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "FOO_ERROR".to_string());
    }
}
