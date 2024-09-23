use crate::string::base64_encode_trunc_of_u8_arr;
use crate::{
    context::ErrCtx,
    error::BoxError,
    hash::hash_sha256_of_str_arr,
    nodebug::NoDebug,
    string::{interpolated_localized_msg, interpolated_string},
};
use serde::Serialize;
use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Write},
    marker::PhantomData,
};

#[derive(Debug)]
pub struct ErrorKind<const ARITY: usize, const HASCAUSE: bool> {
    core: KindCore,
    parent: Option<&'static KindCore>,
}

#[derive(Debug, Serialize)]
pub struct KindCore {
    name: &'static str,
    dev_msg: &'static str,
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

impl<const ARITY: usize, const HASCAUSE: bool> ErrorKind<ARITY, HASCAUSE> {
    pub const fn core(&self) -> &KindCore {
        &self.core
    }

    pub const fn parent(&self) -> Option<&'static KindCore> {
        self.parent
    }

    pub const fn new(
        name: &'static str,
        dev_msg: &'static str,
        parent: Option<&'static KindCore>,
    ) -> Self {
        Self {
            core: KindCore { name, dev_msg },
            parent,
        }
    }

    fn new_error_priv<CTX>(
        &'static self,
        args: [&str; ARITY],
        cause: Option<BoxError>,
    ) -> FoaError<CTX> {
        let args_vec = args
            .into_iter()
            .map(|arg| arg.to_owned())
            .collect::<Vec<_>>();

        FoaError {
            core: self.core(),
            args: args_vec,
            source: cause,
            _ctx: NoDebug(PhantomData),
        }
    }
}

impl ErrorKind<0, false> {
    pub fn new_error<CTX>(&'static self) -> FoaError<CTX> {
        self.new_error_priv([], None)
    }
}

impl ErrorKind<0, true> {
    pub fn new_error<CTX>(
        &'static self,
        cause: impl StdError + Send + Sync + 'static,
    ) -> FoaError<CTX> {
        self.new_error_priv([], Some(BoxError::new_std(cause)))
    }

    pub fn new_error_ser<CTX>(
        &'static self,
        cause: impl StdError + Serialize + Send + Sync + 'static,
    ) -> FoaError<CTX> {
        self.new_error_priv([], Some(BoxError::new_ser(cause)))
    }
}

impl<const ARITY: usize> ErrorKind<ARITY, false> {
    pub fn new_error_with_args<CTX>(&'static self, args: [&str; ARITY]) -> FoaError<CTX> {
        self.new_error_priv(args, None)
    }
}

impl<const ARITY: usize> ErrorKind<ARITY, true> {
    pub fn new_error_with_args<CTX>(
        &'static self,
        args: [&str; ARITY],
        cause: impl StdError + Send + Sync + 'static,
    ) -> FoaError<CTX> {
        self.new_error_priv(args, Some(BoxError::new_std(cause)))
    }

    pub fn new_error_with_args_ser<CTX>(
        &'static self,
        args: [&str; ARITY],
        cause: impl StdError + Serialize + Send + Sync + 'static,
    ) -> FoaError<CTX> {
        self.new_error_priv(args, Some(BoxError::new_ser(cause)))
    }
}

#[derive(Serialize)]
pub struct FoaError<CTX> {
    core: &'static KindCore,
    args: Vec<String>,
    source: Option<BoxError>,
    #[serde(skip_serializing)]
    _ctx: NoDebug<PhantomData<CTX>>,
}

pub type BasicError = FoaError<()>;

#[derive(Debug)]
#[allow(unused)]
struct FoaErrorDevProxy<'a> {
    core: &'a KindCore,
    args: &'a Vec<String>,
    source: &'a Option<BoxError>,
}

#[derive(Debug)]
#[allow(unused)]
struct FoaErrorProdProxy {
    name: &'static str,
    dev_msg: &'static str,
    args: Vec<String>,
}

impl<CTX> FoaError<CTX> {
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
            args: &self.args,
            source: &self.source,
        }
    }

    fn prod_proxy(&self) -> FoaErrorProdProxy {
        let args = self
            .args
            .iter()
            .map(|arg| {
                let hash = hash_sha256_of_str_arr(&[arg]);
                base64_encode_trunc_of_u8_arr(&hash, 8)
            })
            .collect::<Vec<_>>();
        FoaErrorProdProxy {
            name: self.core.name,
            dev_msg: self.core.dev_msg,
            args,
        }
    }
}

impl<CTX> Debug for FoaError<CTX>
where
    CTX: ErrCtx,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            f.write_str("FoaError[")?;
            self.dev_proxy().fmt(f)?;
            f.write_char(']')
        } else {
            f.write_str("FoaError[")?;
            self.prod_proxy().fmt(f)?;
            f.write_str(", display=[")?;
            let msg = interpolated_localized_msg::<CTX, _>(self.core.name, &self.args);
            f.write_str(&msg)?;
            f.write_str("]]")
        }
    }
}

impl<CTX> Display for FoaError<CTX>
where
    CTX: ErrCtx,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            let msg = interpolated_string(self.core.dev_msg, &self.args);
            f.write_str(&msg)
        } else {
            let msg = interpolated_localized_msg::<CTX, _>(self.core.name, &self.args);
            f.write_str(&msg)
        }
    }
}

impl<CTX> StdError for FoaError<CTX>
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

#[cfg(test)]
mod test {
    use super::ErrorKind;

    const FOO_ERROR: ErrorKind<0, false> = ErrorKind::new("FOO_ERROR", "foo message", None);

    #[test]
    fn test() {
        let err = FOO_ERROR.new_error::<()>();
        assert!(err.has_kind(FOO_ERROR));
    }
}
