use std::fmt::Debug;

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
    fn localized_msg<'a>(kind: &'a str, locale: &'a str) -> Option<&'a str>;
}

pub trait Locale {
    fn locale<'a>() -> &'a str;
}

pub trait ErrCtx: Debug + Send + Sync + 'static {
    type Locale: Locale;
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
    fn localized_msg<'a>(_kind: &'a str, _locale: &'a str) -> Option<&'a str> {
        None
    }
}

impl Locale for NullCtxTypeI {
    fn locale<'a>() -> &'a str {
        ""
    }
}

impl ErrCtx for NullCtx {
    type Locale = NullCtxTypeI;
    type LocalizedMsg = NullCtxTypeI;
}
