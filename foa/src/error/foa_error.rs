use super::{JsonBoxError, JsonError, StdBoxError};
use crate::string::base64_encode_trunc_of_u8_arr;
use crate::{error::BoxError, hash::hash_sha256_of_str_arr, string::interpolated_string_props};
use serde::Serialize;
use std::{
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
// region:      --- KindCore

#[derive(Debug, Serialize)]
pub struct KindId(pub &'static str);

impl KindId {
    fn address(&self) -> usize {
        self as *const Self as usize
    }
}

impl PartialEq for KindId {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Eq for KindId {}

// endregion:   --- KindCore

//===========================
// region:      --- PropsError

pub struct PropsError {
    kind_id: &'static KindId,
    msg: &'static str,
    props: Vec<(String, String)>,
}

#[derive(Debug)]
#[allow(unused)]
struct PropsErrorDevProxy<'a> {
    kind_id: &'a KindId,
    msg: &'static str,
    props: &'a Vec<(String, String)>,
}

#[derive(Debug)]
#[allow(unused)]
struct PropsErrorProdProxy {
    kind_id: &'static KindId,
    msg: &'static str,
    props: Vec<(String, String)>,
}

impl PropsError {
    fn dev_proxy(&self) -> PropsErrorDevProxy {
        PropsErrorDevProxy {
            kind_id: self.kind_id,
            msg: self.msg,
            props: &self.props,
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
            kind_id: self.kind_id,
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

impl std::error::Error for PropsError {}

// endregion:   --- PropsError

//===========================
// region:      --- PropsErrorKind

#[derive(Debug)]
pub struct PropsErrorKind<const ARITY: usize, const HASCAUSE: bool> {
    kind_id: KindId,
    msg: &'static str,
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
        msg: &'static str,
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
        let payload = PropsError {
            kind_id: &self.kind_id(),
            msg: self.msg,
            props,
        };

        Error {
            kind_id: self.kind_id(),
            payload: StdBoxError::new(payload),
            source: cause,
        }
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

//===========================
// region:      --- Error

#[derive(Debug, Serialize)]
pub struct Error {
    kind_id: &'static KindId,
    payload: StdBoxError,
    source: Option<StdBoxError>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[allow(unused)]
struct FoaErrorDevProxy<'a> {
    kind_id: &'a KindId,
    props: &'a Vec<(String, String)>,
    source: &'a Option<BoxError>,
}

#[derive(Debug)]
#[allow(unused)]
struct FoaErrorProdProxy {
    name: &'static str,
    msg: &'static str,
    props: Vec<(String, String)>,
}

impl Error {
    pub fn new<C>(
        kind_id: &'static KindId,
        payload: impl StdError + Send + Sync + 'static,
        cause: Option<C>,
    ) -> Self
    where
        C: StdError + Send + Sync + 'static,
    {
        Self {
            kind_id,
            payload: StdBoxError::new(payload),
            source: cause.map(|c| StdBoxError::new(c)),
        }
    }

    pub fn has_kind(&self, kind: &'static KindId) -> bool {
        self.kind_id == kind
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.payload, f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.source {
            Some(source) => {
                let err = source.as_dyn_std_error();
                Some(err)
            }
            None => None,
        }
    }
}

impl From<Error> for Box<dyn JsonError> {
    fn from(value: Error) -> Self {
        Box::new(value)
    }
}

impl From<Error> for JsonBoxError {
    fn from(value: Error) -> Self {
        JsonBoxError::new(value)
    }
}

// endregion:   --- ErrorKind

#[cfg(test)]
mod test {
    use super::PropsErrorKind;

    const FOO_ERROR: PropsErrorKind<0, false> =
        PropsErrorKind::new("FOO_ERROR", "foo message", [], None);

    #[test]
    fn test() {
        let err = FOO_ERROR.new_error();
        assert!(err.has_kind(FOO_ERROR.kind_id()));
    }
}
