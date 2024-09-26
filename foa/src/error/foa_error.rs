use crate::string::base64_encode_trunc_of_u8_arr;
use crate::{
    context::ErrCtx,
    error::BoxError,
    hash::hash_sha256_of_str_arr,
    nodebug::NoDebug,
    string::{interpolated_localized_msg_props, interpolated_string_props},
};
use serde::Serialize;
use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
    marker::PhantomData,
};

use super::{SerBoxError, SerializableError};

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ErrorTag(pub &'static str);

#[derive(Debug, Serialize)]
pub struct KindCore {
    name: &'static str,
    msg: &'static str,
}

impl KindCore {
    fn address(&self) -> usize {
        self as *const Self as usize
    }
}

impl PartialEq for KindCore {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Eq for KindCore {}

#[derive(Debug)]
pub struct ErrorKind<const ARITY: usize, const HASCAUSE: bool> {
    core: KindCore,
    prop_names: [&'static str; ARITY],
    tag: Option<&'static ErrorTag>,
}

impl<const ARITY: usize, const HASCAUSE: bool> ErrorKind<ARITY, HASCAUSE> {
    pub const fn core(&self) -> &KindCore {
        &self.core
    }

    pub const fn tag(&self) -> Option<&'static ErrorTag> {
        self.tag
    }

    pub const fn new(
        name: &'static str,
        msg: &'static str,
        field_names: [&'static str; ARITY],
        tag: Option<&'static ErrorTag>,
    ) -> Self {
        Self {
            core: KindCore { name, msg },
            prop_names: field_names,
            tag,
        }
    }

    fn new_error_priv<CTX>(
        &'static self,
        args: [&str; ARITY],
        cause: Option<BoxError>,
    ) -> Error<CTX> {
        let props = args
            .into_iter()
            .zip(self.prop_names)
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect::<Vec<_>>();

        Error {
            core: self.core(),
            props,
            source: cause,
            _ctx: NoDebug(PhantomData),
        }
    }
}

impl ErrorKind<0, false> {
    pub fn new_error<CTX>(&'static self) -> Error<CTX> {
        self.new_error_priv([], None)
    }
}

impl ErrorKind<0, true> {
    pub fn new_error<CTX>(
        &'static self,
        cause: impl StdError + Send + Sync + 'static,
    ) -> Error<CTX> {
        self.new_error_priv([], Some(BoxError::new_std(cause)))
    }

    pub fn new_error_ser<CTX>(
        &'static self,
        cause: impl StdError + Serialize + Send + Sync + 'static,
    ) -> Error<CTX> {
        self.new_error_priv([], Some(BoxError::new_ser(cause)))
    }
}

impl<const ARITY: usize> ErrorKind<ARITY, false> {
    pub fn new_error_with_args<CTX>(&'static self, args: [&str; ARITY]) -> Error<CTX> {
        self.new_error_priv(args, None)
    }
}

impl<const ARITY: usize> ErrorKind<ARITY, true> {
    pub fn new_error_with_args<CTX>(
        &'static self,
        args: [&str; ARITY],
        cause: impl StdError + Send + Sync + 'static,
    ) -> Error<CTX> {
        self.new_error_priv(args, Some(BoxError::new_std(cause)))
    }

    pub fn new_error_with_args_ser<CTX>(
        &'static self,
        args: [&str; ARITY],
        cause: impl StdError + Serialize + Send + Sync + 'static,
    ) -> Error<CTX> {
        self.new_error_priv(args, Some(BoxError::new_ser(cause)))
    }
}

#[derive(Serialize)]
pub struct Error<CTX> {
    core: &'static KindCore,
    props: Vec<(String, String)>,
    source: Option<BoxError>,
    #[serde(skip_serializing)]
    _ctx: NoDebug<PhantomData<CTX>>,
}

pub type BasicError = Error<()>;

#[derive(Debug)]
#[allow(unused)]
struct FoaErrorDevProxy<'a> {
    core: &'a KindCore,
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

impl<CTX> Error<CTX> {
    pub fn new_error<const ARITY: usize, const HASCAUSE: bool>(
        kind: &'static ErrorKind<ARITY, HASCAUSE>,
        args: [&str; ARITY],
        cause: Option<BoxError>,
    ) -> Self {
        kind.new_error_priv(args, cause)
    }

    #[deprecated]
    pub fn new(kind: &'static ErrorKind<0, false>) -> Self {
        Self::new_error(kind, [], None)
    }

    #[deprecated]
    pub fn new_with_args<const ARITY: usize>(
        kind: &'static ErrorKind<ARITY, false>,
        args: [&str; ARITY],
    ) -> Self {
        Self::new_error(kind, args, None)
    }

    #[deprecated]
    pub fn new_with_cause(
        kind: &'static ErrorKind<0, true>,
        cause: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::new_error(kind, [], Some(BoxError::new_std(cause)))
    }

    #[deprecated]
    pub fn new_with_cause_ser(
        kind: &'static ErrorKind<0, true>,
        cause: impl StdError + Serialize + Send + Sync + 'static,
    ) -> Self {
        Self::new_error(kind, [], Some(BoxError::new_ser(cause)))
    }

    #[deprecated]
    pub fn new_with_args_and_cause<const ARITY: usize>(
        kind: &'static ErrorKind<ARITY, true>,
        args: [&str; ARITY],
        cause: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::new_error(kind, args, Some(BoxError::new_std(cause)))
    }

    #[deprecated]
    pub fn new_with_args_and_cause_ser<const ARITY: usize>(
        kind: &'static ErrorKind<ARITY, true>,
        args: [&str; ARITY],
        cause: impl StdError + Serialize + Send + Sync + 'static,
    ) -> Self {
        Self::new_error(kind, args, Some(BoxError::new_ser(cause)))
    }

    pub fn has_kind<const A: usize, const H: bool>(&self, kind: ErrorKind<A, H>) -> bool {
        self.core == kind.core()
    }

    fn dev_proxy(&self) -> FoaErrorDevProxy {
        FoaErrorDevProxy {
            core: &self.core,
            props: &self.props,
            source: &self.source,
        }
    }

    fn prod_proxy(&self) -> FoaErrorProdProxy {
        let fields = self
            .props
            .iter()
            .map(|(name, value)| {
                let vhash = hash_sha256_of_str_arr(&[value]);
                let vb64 = base64_encode_trunc_of_u8_arr(&vhash, 8);
                (name.to_owned(), vb64)
            })
            .collect::<Vec<_>>();
        FoaErrorProdProxy {
            name: self.core.name,
            msg: self.core.msg,
            props: fields,
        }
    }
}

impl<CTX> Debug for Error<CTX>
where
    CTX: ErrCtx,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            self.dev_proxy().fmt(f)
        } else {
            self.prod_proxy().fmt(f)
        }
    }
}

impl<CTX> Display for Error<CTX>
where
    CTX: ErrCtx,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            let msg =
                interpolated_string_props(self.core.msg, self.props.iter().map(|p| (&p.0, &p.1)));
            f.write_str(&msg)
        } else {
            let msg = interpolated_localized_msg_props::<CTX, _, _>(
                self.core.name,
                self.props.iter().map(|p| (&p.0, &p.1)),
            );
            f.write_str(&msg)
        }
    }
}

impl<CTX> StdError for Error<CTX>
where
    CTX: ErrCtx,
{
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

impl<CTX: ErrCtx> From<Error<CTX>> for Box<dyn SerializableError> {
    fn from(value: Error<CTX>) -> Self {
        Box::new(value)
    }
}

impl<CTX: ErrCtx> From<Error<CTX>> for SerBoxError {
    fn from(value: Error<CTX>) -> Self {
        SerBoxError::new(value)
    }
}

#[cfg(test)]
mod test {
    use super::ErrorKind;

    const FOO_ERROR: ErrorKind<0, false> = ErrorKind::new("FOO_ERROR", "foo message", [], None);

    #[test]
    fn test() {
        let err = FOO_ERROR.new_error::<()>();
        assert!(err.has_kind(FOO_ERROR));
    }
}
