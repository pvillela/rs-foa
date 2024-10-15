use super::{BacktraceSpec, Error, KindId, StdBoxError, Tag, TRUNC};
use crate::{hash::hash_sha256_of_str_arr, string};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::{error::Error as StdError, fmt::Debug};

//===========================
// region:      --- Props

#[derive(Deserialize, Clone, PartialEq, Eq)]
pub struct Props {
    pub(crate) props: Vec<(String, String)>,
    pub(crate) sensitive: bool,
    pub(crate) locked: bool,
}

impl Props {
    pub fn props(&self) -> impl Iterator<Item = (&str, &str)> {
        self.props.iter().map(|p| (p.0.as_str(), p.1.as_str()))
    }

    pub fn prop(&self, key: &str) -> Option<&str> {
        self.props
            .iter()
            .find(|&p| p.0 == key)
            .map(|p| p.1.as_str())
    }

    fn hashed_props(&self) -> Vec<(String, String)> {
        self.props
            .iter()
            .map(|(name, value)| {
                let vhash = hash_sha256_of_str_arr(&[value]);
                let vb64 = string::base64_encode_trunc_of_u8_arr(&vhash, TRUNC);
                (name.to_owned(), vb64)
            })
            .collect::<Vec<_>>()
    }

    pub fn safe_props(&self) -> Self {
        if cfg!(debug_assertions) || !self.sensitive || self.locked {
            self.clone()
        } else {
            Self {
                props: self.hashed_props(),
                sensitive: self.sensitive,
                locked: true,
            }
        }
    }
}

impl Debug for Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let props = if cfg!(debug_assertions) || !self.sensitive || self.locked {
            &self.props
        } else {
            &self.hashed_props()
        };
        f.write_str("Props { props: ")?;
        props.fmt(f)?;
        f.write_str(", sensitive: ")?;
        self.sensitive.fmt(f)?;
        f.write_str(", locked: ")?;
        self.locked.fmt(f)?;
        f.write_str(" }")
    }
}

impl Serialize for Props {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (props, locked) = if cfg!(debug_assertions) || !self.sensitive || self.locked {
            (&self.props, self.locked)
        } else {
            (&self.hashed_props(), true)
        };
        let mut state = serializer.serialize_struct("Props", 3)?;
        state.serialize_field("props", &props)?;
        state.serialize_field("sensitive", &self.sensitive)?;
        state.serialize_field("locked", &locked)?;
        state.end()
    }
}

// endregion:   --- Props

//===========================
// region:      --- PropsKind

#[derive(Debug)]
pub struct PropsKind<const ARITY: usize, const HASCAUSE: bool, const SENSITIVE: bool = true> {
    pub(super) kind_id: KindId,
    pub(super) msg: Option<&'static str>,
    pub(super) prop_names: [&'static str; ARITY],
    pub(super) backtrace_spec: BacktraceSpec,
    pub(super) tag: &'static Tag,
}

pub type BasicKind<const HASCAUSE: bool> = PropsKind<0, HASCAUSE, false>;

impl<const ARITY: usize, const HASCAUSE: bool, const SENSITIVE: bool>
    PropsKind<ARITY, HASCAUSE, SENSITIVE>
{
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

    pub const fn with_prop_names(
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
        }
    }

    fn error_priv(&'static self, values: [&str; ARITY], source: Option<StdBoxError>) -> Error {
        let props = self
            .prop_names
            .into_iter()
            .zip(values)
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect::<Vec<_>>();
        let msg = match self.msg {
            Some(msg) => msg,
            None => self.kind_id.0,
        };
        let payload = Props {
            props,
            sensitive: SENSITIVE,
            locked: false,
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
            payload,
            source,
            backtrace,
        )
    }
}

impl<const HASCAUSE: bool> BasicKind<HASCAUSE> {
    pub const fn new(
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
        }
    }
}

impl BasicKind<false> {
    pub fn error(&'static self) -> Error {
        self.error_priv([], None)
    }
}

impl BasicKind<true> {
    pub fn error(&'static self, cause: impl StdError + Send + Sync + 'static) -> Error {
        self.error_priv([], Some(StdBoxError::new(cause)))
    }
}

impl<const ARITY: usize, const SENSITIVE: bool> PropsKind<ARITY, false, SENSITIVE> {
    pub fn error_with_values(&'static self, values: [&str; ARITY]) -> Error {
        self.error_priv(values, None)
    }
}

impl<const ARITY: usize, const SENSITIVE: bool> PropsKind<ARITY, true, SENSITIVE> {
    pub fn error_with_values(
        &'static self,
        values: [&str; ARITY],
        cause: impl StdError + Send + Sync + 'static,
    ) -> Error {
        self.error_priv(values, Some(StdBoxError::new(cause)))
    }
}

// endregion:   --- PropsKind

#[cfg(test)]
mod test_props_error {
    use super::PropsKind;
    use crate::error::{BacktraceSpec, Tag};

    static FOO_TAG: Tag = Tag("FOO");

    static FOO_ERROR: PropsKind<1, false> = PropsKind::with_prop_names(
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
mod test_basic_error {
    use super::BasicKind;
    use crate::error::{BacktraceSpec, Tag};

    static FOO_TAG: Tag = Tag("FOO");

    static FOO_ERROR: BasicKind<false> =
        BasicKind::new("FOO_ERROR", None, BacktraceSpec::Env, &FOO_TAG);

    #[test]
    fn test() {
        let err = FOO_ERROR.error();
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "FOO_ERROR");
    }
}
