use std::{fmt::Debug, ops::Deref};

//=============
// Context traits

pub trait Itself<CTX> {
    fn itself() -> CTX;
}

pub trait Cfg {
    type CfgInfo;

    fn cfg() -> Self::CfgInfo;
}

pub trait LocalizedMsg {
    fn localized_msg<'a>(kind: &'a str, locale: impl Deref<Target = str>) -> Option<&'a str>;
}

pub trait Locale {
    fn locale() -> impl Deref<Target = str> + Send;
}

pub trait LocaleSelf {
    fn locale(&self) -> &str;
}

pub trait LocaleCtx {
    type Locale: Locale;
}

pub trait ErrCtx: LocaleCtx + Debug + Send + Sync + 'static {
    type LocalizedMsg: LocalizedMsg;
}

pub trait SecCtx {
    // TBD
}

//=============
// impls for NullCtx

pub type NullCtx = ();

pub struct NullCtxTypeI;

impl LocalizedMsg for NullCtxTypeI {
    fn localized_msg<'a>(_kind: &'a str, _locale: impl Deref<Target = str>) -> Option<&'a str> {
        None
    }
}

impl Locale for NullCtxTypeI {
    fn locale() -> impl Deref<Target = str> {
        ""
    }
}

impl LocaleCtx for NullCtx {
    type Locale = NullCtxTypeI;
}

impl ErrCtx for NullCtx {
    type LocalizedMsg = NullCtxTypeI;
}
