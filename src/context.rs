//=============
// Context traits

use std::fmt::Debug;

pub trait Cfg {
    type Info;

    fn cfg() -> Self::Info;
}

pub trait CfgCtx {
    type Cfg: Cfg;
}
pub trait LocalizedMsg {
    fn localized_msg<'a>(kind: &'a str, locale: &'a str) -> Option<&'a str>;
}

pub trait Locale {
    fn locale<'a>() -> &'a str;
}

pub trait ErrCtx: Debug {
    type Locale: Locale;
    type LocalizedMsg: LocalizedMsg;
}

pub trait DbCtx {
    type Db;
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
