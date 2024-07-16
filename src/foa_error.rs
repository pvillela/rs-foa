use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Write},
    marker::PhantomData,
};

use crate::{
    interpolated_localized_msg, interpolated_string, Locale, LocalizedMsg, NoDebug, NullCtx,
};

#[derive(Debug)]
pub struct ErrorKind {
    name: &'static str,
    dev_msg: &'static str,
    arity: usize,
    has_cause: bool,
}

impl ErrorKind {
    const fn new_priv(
        name: &'static str,
        debug_msg: &'static str,
        arity: usize,
        has_cause: bool,
    ) -> Self {
        Self {
            name,
            dev_msg: debug_msg,
            arity,
            has_cause,
        }
    }

    pub const fn new(name: &'static str, debug_msg: &'static str) -> Self {
        Self::new_priv(name, debug_msg, 0, false)
    }

    pub const fn new_with_args(name: &'static str, debug_msg: &'static str, arity: usize) -> Self {
        Self::new_priv(name, debug_msg, arity, false)
    }

    pub const fn new_with_cause(name: &'static str, debug_msg: &'static str) -> Self {
        Self::new_priv(name, debug_msg, 0, true)
    }

    pub const fn new_with_args_and_cause(
        name: &'static str,
        debug_msg: &'static str,
        arity: usize,
    ) -> Self {
        Self::new_priv(name, debug_msg, arity, true)
    }
}

#[derive(Debug)]
pub struct FoaError<CTX = NullCtx> {
    kind: &'static ErrorKind,
    args: Vec<String>,
    source: Option<Box<dyn StdError + 'static>>,
    _ctx: NoDebug<PhantomData<CTX>>,
}

impl FoaError {
    fn new_priv<const N: usize>(
        kind: &'static ErrorKind,
        args: [&str; N],
        cause: Option<Box<dyn StdError>>,
    ) -> Option<Self> {
        let args_vec = args
            .into_iter()
            .map(|arg| arg.to_owned())
            .collect::<Vec<_>>();
        let err = Self {
            kind,
            args: args_vec,
            source: cause,
            _ctx: NoDebug(PhantomData),
        };

        Some(err)
    }

    pub fn new(kind: &'static ErrorKind) -> Option<Self> {
        if kind.arity != 0 || kind.has_cause {
            return None;
        }
        Self::new_priv(kind, [], None)
    }

    pub fn new_with_args<const N: usize>(
        kind: &'static ErrorKind,
        args: [&str; N],
    ) -> Option<Self> {
        if kind.arity != N || kind.has_cause {
            return None;
        }
        Self::new_priv(kind, args, None)
    }

    pub fn new_with_cause(
        kind: &'static ErrorKind,
        cause: impl StdError + 'static,
    ) -> Option<Self> {
        if kind.arity != 0 || !kind.has_cause {
            return None;
        }
        Self::new_priv(kind, [], Some(Box::new(cause)))
    }

    pub fn new_with_args_and_cause<const N: usize>(
        kind: &'static ErrorKind,
        args: [&str; N],
        cause: impl StdError + 'static,
    ) -> Option<Self> {
        if kind.arity != N || !kind.has_cause {
            return None;
        }
        Self::new_priv(kind, args, Some(Box::new(cause)))
    }
}

impl<CTX> Display for FoaError<CTX>
where
    CTX: LocalizedMsg + Locale + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            let msg = interpolated_string(&self.kind.dev_msg, &self.args);
            f.write_str(&msg)?;
            f.write_str(" [")?;
            <Self as Debug>::fmt(self, f)?;
            f.write_char(']')
        } else {
            let msg = interpolated_localized_msg::<CTX>(self.kind.name, &self.args);
            f.write_str(&msg)
        }
    }
}

impl StdError for FoaError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.source {
            Some(source) => {
                let err = &*source;
                err.source()
            }
            None => None,
        }
    }
}
