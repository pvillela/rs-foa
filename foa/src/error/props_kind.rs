use super::{BacktraceSpec, Error, ErrorTag, KindId, StdBoxError, TRUNC};
use crate::string::base64_encode_trunc_of_u8_arr;
use crate::{hash::hash_sha256_of_str_arr, string::interpolated_string_props};
use serde::Serialize;
use std::backtrace::Backtrace;
use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
};

//===========================
// region:      --- PropsError

#[derive(Serialize)]
pub struct PropsError {
    pub(crate) msg: &'static str,
    pub(crate) props: Vec<(String, String)>,
    pub(crate) source: Option<StdBoxError>,
}

#[derive(Debug)]
#[allow(unused)]
struct PropsErrorDev<'a> {
    msg: &'static str,
    props: &'a Vec<(String, String)>,
    source: &'a Option<StdBoxError>,
}

#[derive(Debug)]
#[allow(unused)]
struct PropsErrorProd<'a> {
    msg: &'static str,
    props: Vec<(String, String)>,
    source: &'a Option<StdBoxError>,
}

impl PropsError {
    pub fn props(&self) -> impl Iterator<Item = (&str, &str)> {
        self.props.iter().map(|p| (p.0.as_str(), p.1.as_str()))
    }

    pub fn prop(&self, key: &str) -> Option<&str> {
        self.props
            .iter()
            .find(|&p| p.0 == key)
            .map(|p| p.1.as_str())
    }

    fn dev_proxy(&self) -> PropsErrorDev {
        PropsErrorDev {
            msg: self.msg,
            props: &self.props,
            source: &self.source,
        }
    }

    fn prod_proxy(&self) -> PropsErrorProd {
        let props = self
            .props
            .iter()
            .map(|(name, value)| {
                let vhash = hash_sha256_of_str_arr(&[value]);
                let vb64 = base64_encode_trunc_of_u8_arr(&vhash, TRUNC);
                (name.to_owned(), vb64)
            })
            .collect::<Vec<_>>();
        PropsErrorProd {
            msg: self.msg,
            props,
            source: &self.source,
        }
    }
}

impl Debug for PropsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            self.dev_proxy().fmt(f)
        } else {
            self.prod_proxy().fmt(f)
        }
    }
}

impl Display for PropsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            let msg = interpolated_string_props(self.msg, self.props.iter().map(|p| (&p.0, &p.1)));
            f.write_str(&msg)
        } else {
            let props = self.props.iter().map(|p| {
                let (name, value) = (&p.0, &p.1);
                let vhash = hash_sha256_of_str_arr(&[value]);
                let vb64 = base64_encode_trunc_of_u8_arr(&vhash, TRUNC);
                (name, vb64)
            });
            let msg = interpolated_string_props(self.msg, props);
            f.write_str(&msg)
        }
    }
}

impl std::error::Error for PropsError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.source {
            Some(e) => Some(e.as_dyn_std_error()),
            None => None,
        }
    }
}

// endregion:   --- PropsError

//===========================
// region:      --- PropsErrorKind

#[derive(Debug)]
pub struct PropsErrorKind<const ARITY: usize, const HASCAUSE: bool> {
    pub(super) kind_id: KindId,
    pub(super) msg: Option<&'static str>,
    pub(super) prop_names: [&'static str; ARITY],
    pub(super) backtrace_spec: BacktraceSpec,
    pub(super) tag: Option<&'static ErrorTag>,
}

pub type BasicErrorKind<const HASCAUSE: bool> = PropsErrorKind<0, HASCAUSE>;

impl<const ARITY: usize, const HASCAUSE: bool> PropsErrorKind<ARITY, HASCAUSE> {
    pub const fn kind_id(&self) -> &KindId {
        &self.kind_id
    }

    pub const fn tag(&self) -> Option<&'static ErrorTag> {
        self.tag
    }

    pub const fn with_prop_names(
        name: &'static str,
        msg: Option<&'static str>,
        prop_names: [&'static str; ARITY],
        backtrace_spec: BacktraceSpec,
        tag: Option<&'static ErrorTag>,
    ) -> Self {
        Self {
            kind_id: KindId(name),
            msg,
            prop_names,
            backtrace_spec,
            tag,
        }
    }

    fn error_priv(&'static self, values: [&str; ARITY], cause: Option<StdBoxError>) -> Error {
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
        let payload = PropsError {
            msg,
            props,
            source: cause,
        };
        let backtrace = match self.backtrace_spec {
            BacktraceSpec::Yes => Backtrace::force_capture(),
            BacktraceSpec::No => Backtrace::disabled(),
            BacktraceSpec::Env => Backtrace::capture(),
        };

        Error::new(self.kind_id(), self.tag, payload, backtrace)
    }
}

impl<const HASCAUSE: bool> BasicErrorKind<HASCAUSE> {
    pub const fn new(
        name: &'static str,
        msg: Option<&'static str>,
        backtrace_spec: BacktraceSpec,
        tag: Option<&'static ErrorTag>,
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

impl BasicErrorKind<false> {
    pub fn error(&'static self) -> Error {
        self.error_priv([], None)
    }
}

impl BasicErrorKind<true> {
    pub fn error(&'static self, cause: impl StdError + Send + Sync + 'static) -> Error {
        self.error_priv([], Some(StdBoxError::new(cause)))
    }
}

impl<const ARITY: usize> PropsErrorKind<ARITY, false> {
    pub fn error_with_values(&'static self, values: [&str; ARITY]) -> Error {
        self.error_priv(values, None)
    }
}

impl<const ARITY: usize> PropsErrorKind<ARITY, true> {
    pub fn error_with_values(
        &'static self,
        values: [&str; ARITY],
        cause: impl StdError + Send + Sync + 'static,
    ) -> Error {
        self.error_priv(values, Some(StdBoxError::new(cause)))
    }
}

// endregion:   --- PropsErrorKind

#[cfg(test)]
mod test_props_error {
    use super::PropsErrorKind;
    use crate::error::BacktraceSpec;

    static FOO_ERROR: PropsErrorKind<1, false> = PropsErrorKind::with_prop_names(
        "FOO_ERROR",
        Some("foo message: {xyz}"),
        ["xyz"],
        BacktraceSpec::Env,
        None,
    );

    #[test]
    fn test() {
        let err = FOO_ERROR.error_with_values(["hi there!"]);
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message: hi there!");
    }
}

#[cfg(test)]
mod test_basic_error {
    use super::BasicErrorKind;
    use crate::error::BacktraceSpec;

    static FOO_ERROR: BasicErrorKind<false> =
        BasicErrorKind::new("FOO_ERROR", None, BacktraceSpec::Env, None);

    #[test]
    fn test() {
        let err = FOO_ERROR.error();
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "FOO_ERROR");
    }
}
