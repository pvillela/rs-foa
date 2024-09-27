use super::{Error, ErrorTag, KindId, StdBoxError, TRUNC};
use crate::string::base64_encode_trunc_of_u8_arr;
use crate::{hash::hash_sha256_of_str_arr, string::interpolated_string_props};
use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
};

//===========================
// region:      --- PropsError

pub struct PropsError {
    msg: &'static str,
    props: Vec<(String, String)>,
    source: Option<StdBoxError>,
}

#[derive(Debug)]
#[allow(unused)]
struct PropsErrorDevProxy<'a> {
    msg: &'static str,
    props: &'a Vec<(String, String)>,
    source: &'a Option<StdBoxError>,
}

#[derive(Debug)]
#[allow(unused)]
struct PropsErrorProdProxy {
    msg: &'static str,
    props: Vec<(String, String)>,
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

    fn dev_proxy(&self) -> PropsErrorDevProxy {
        PropsErrorDevProxy {
            msg: self.msg,
            props: &self.props,
            source: &self.source,
        }
    }

    fn prod_proxy(&self) -> PropsErrorProdProxy {
        let props = self
            .props
            .iter()
            .map(|(name, value)| {
                let vhash = hash_sha256_of_str_arr(&[value]);
                let vb64 = base64_encode_trunc_of_u8_arr(&vhash, TRUNC);
                (name.to_owned(), vb64)
            })
            .collect::<Vec<_>>();
        PropsErrorProdProxy {
            msg: self.msg,
            props,
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
    kind_id: KindId,
    msg: Option<&'static str>,
    prop_names: [&'static str; ARITY],
    tag: Option<&'static ErrorTag>,
}

impl<const ARITY: usize, const HASCAUSE: bool> PropsErrorKind<ARITY, HASCAUSE> {
    pub const fn kind_id(&self) -> &KindId {
        &self.kind_id
    }

    pub const fn tag(&self) -> Option<&'static ErrorTag> {
        self.tag
    }

    pub const fn new(
        name: &'static str,
        msg: Option<&'static str>,
        prop_names: [&'static str; ARITY],
        tag: Option<&'static ErrorTag>,
    ) -> Self {
        Self {
            kind_id: KindId(name),
            msg,
            prop_names,
            tag,
        }
    }

    fn new_error_priv(&'static self, args: [&str; ARITY], cause: Option<StdBoxError>) -> Error {
        let props = self
            .prop_names
            .into_iter()
            .zip(args)
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

        Error::new(self.kind_id(), self.tag, payload)
    }
}

impl PropsErrorKind<0, false> {
    pub fn new_error(&'static self) -> Error {
        self.new_error_priv([], None)
    }
}

impl PropsErrorKind<0, true> {
    pub fn new_error(&'static self, cause: impl StdError + Send + Sync + 'static) -> Error {
        self.new_error_priv([], Some(StdBoxError::new(cause)))
    }
}

impl<const ARITY: usize> PropsErrorKind<ARITY, false> {
    pub fn new_error_with_args(&'static self, args: [&str; ARITY]) -> Error {
        self.new_error_priv(args, None)
    }
}

impl<const ARITY: usize> PropsErrorKind<ARITY, true> {
    pub fn new_error_with_args(
        &'static self,
        args: [&str; ARITY],
        cause: impl StdError + Send + Sync + 'static,
    ) -> Error {
        self.new_error_priv(args, Some(StdBoxError::new(cause)))
    }
}

// endregion:   --- PropsErrorKind

#[cfg(test)]
mod test {
    use super::PropsErrorKind;

    const FOO_ERROR: PropsErrorKind<0, false> =
        PropsErrorKind::new("FOO_ERROR", Some("foo message"), [], None);

    #[test]
    fn test() {
        let err = FOO_ERROR.new_error();
        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message");
    }
}
