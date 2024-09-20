use crate::{
    context::ErrCtx,
    error::BoxError,
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
pub struct ErrorKind<const ARITY: usize, const HASCAUSE: bool>(
    /// name
    pub &'static str,
    /// dev message
    pub &'static str,
);

#[derive(Debug, Serialize)]
struct Kind {
    name: &'static str,
    dev_msg: &'static str,
}

impl<const ARITY: usize, const HASCAUSE: bool> ErrorKind<ARITY, HASCAUSE> {
    const fn to_uni(&self) -> Kind {
        Kind {
            name: self.0,
            dev_msg: self.1,
        }
    }

    fn new_error_priv<CTX>(&self, args: [&str; ARITY], cause: Option<BoxError>) -> FoaError<CTX> {
        let args_vec = args
            .into_iter()
            .map(|arg| arg.to_owned())
            .collect::<Vec<_>>();

        FoaError {
            kind: self.to_uni(),
            args: args_vec,
            source: cause,
            _ctx: NoDebug(PhantomData),
        }
    }
}

impl ErrorKind<0, false> {
    pub fn new_error<CTX>(&self) -> FoaError<CTX> {
        self.new_error_priv([], None)
    }
}

impl ErrorKind<0, true> {
    pub fn new_error<CTX>(&self, cause: impl StdError + Send + Sync + 'static) -> FoaError<CTX> {
        self.new_error_priv([], Some(BoxError::new_std(cause)))
    }

    pub fn new_error_ser<CTX>(
        &self,
        cause: impl StdError + Serialize + Send + Sync + 'static,
    ) -> FoaError<CTX> {
        self.new_error_priv([], Some(BoxError::new_ser(cause)))
    }
}

impl<const ARITY: usize> ErrorKind<ARITY, false> {
    pub fn new_error_with_args<CTX>(&self, args: [&str; ARITY]) -> FoaError<CTX> {
        self.new_error_priv(args, None)
    }
}

impl<const ARITY: usize> ErrorKind<ARITY, true> {
    pub fn new_error_with_args<CTX>(
        &self,
        args: [&str; ARITY],
        cause: impl StdError + Send + Sync + 'static,
    ) -> FoaError<CTX> {
        self.new_error_priv(args, Some(BoxError::new_std(cause)))
    }

    pub fn new_error_with_args_ser<CTX>(
        &self,
        args: [&str; ARITY],
        cause: impl StdError + Serialize + Send + Sync + 'static,
    ) -> FoaError<CTX> {
        self.new_error_priv(args, Some(BoxError::new_ser(cause)))
    }
}

#[derive(Serialize)]
pub struct FoaError<CTX> {
    kind: Kind,
    args: Vec<String>,
    source: Option<BoxError>,
    #[serde(skip_serializing)]
    _ctx: NoDebug<PhantomData<CTX>>,
}

pub type BasicError = FoaError<()>;

#[derive(Debug)]
#[allow(unused)]
struct FoaErrorDevProxy<'a> {
    kind: &'a Kind,
    args: &'a Vec<String>,
    source: &'a Option<BoxError>,
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
        self.kind.name == kind.0
    }

    fn dev_proxy(&self) -> FoaErrorDevProxy {
        FoaErrorDevProxy {
            kind: &self.kind,
            args: &self.args,
            source: &self.source,
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
            // f.write_str("], display=[")?;
            // let msg = interpolated_string(self.kind.dev_msg, &self.args);
            // f.write_str(&msg)?;
            f.write_char(']')
        } else {
            f.write_str("FoaError[")?;
            self.kind.fmt(f)?;
            f.write_str(", display=[")?;
            let msg = interpolated_localized_msg::<CTX>(self.kind.name, &self.args);
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
            // f.write_str("display=[")?;
            let msg = interpolated_string(self.kind.dev_msg, &self.args);
            f.write_str(&msg)
            // f.write_str("], debug=[")?;
            // <Self as Debug>::fmt(self, f)
            // f.write_char(']')
        } else {
            let msg = interpolated_localized_msg::<CTX>(self.kind.name, &self.args);
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

    const FOO_ERROR: ErrorKind<0, false> = ErrorKind("FOO_ERROR", "foo message");

    #[test]
    fn test() {
        let err = FOO_ERROR.new_error::<()>();
        assert!(err.has_kind(FOO_ERROR));
    }
}
