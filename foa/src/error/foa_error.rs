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
    fn new_priv<const ARITY: usize, const HASCAUSE: bool>(
        kind: &'static ErrorKind<ARITY, HASCAUSE>,
        args: [&str; ARITY],
        cause: Option<BoxError>,
    ) -> Self {
        let args_vec = args
            .into_iter()
            .map(|arg| arg.to_owned())
            .collect::<Vec<_>>();

        Self {
            kind: kind.to_uni(),
            args: args_vec,
            source: cause,
            _ctx: NoDebug(PhantomData),
        }
    }

    pub fn new(kind: &'static ErrorKind<0, false>) -> Self {
        Self::new_priv(kind, [], None)
    }

    pub fn new_with_args<const ARITY: usize>(
        kind: &'static ErrorKind<ARITY, false>,
        args: [&str; ARITY],
    ) -> Self {
        Self::new_priv(kind, args, None)
    }

    pub fn new_with_cause_std(
        kind: &'static ErrorKind<0, true>,
        cause: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::new_priv(kind, [], Some(BoxError::new_std(cause)))
    }

    pub fn new_with_cause_ser(
        kind: &'static ErrorKind<0, true>,
        cause: impl StdError + Serialize + Send + Sync + 'static,
    ) -> Self {
        Self::new_priv(kind, [], Some(BoxError::new_ser(cause)))
    }

    pub fn new_with_args_and_cause_std<const ARITY: usize>(
        kind: &'static ErrorKind<ARITY, true>,
        args: [&str; ARITY],
        cause: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::new_priv(kind, args, Some(BoxError::new_std(cause)))
    }

    pub fn new_with_args_and_cause_ser<const ARITY: usize>(
        kind: &'static ErrorKind<ARITY, true>,
        args: [&str; ARITY],
        cause: impl StdError + Serialize + Send + Sync + 'static,
    ) -> Self {
        Self::new_priv(kind, args, Some(BoxError::new_ser(cause)))
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
    use super::{ErrorKind, FoaError};

    const FOO_ERROR: ErrorKind<0, false> = ErrorKind("FOO_ERROR", "foo message");

    #[test]
    fn test() {
        let err = FoaError::<()>::new(&FOO_ERROR);
        assert!(err.has_kind(FOO_ERROR));
    }
}
