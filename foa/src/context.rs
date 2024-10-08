use std::{fmt::Debug, ops::Deref};

// region:      --- Context traits

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
    fn locale(&self) -> Option<&str>;
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

pub trait Source<T> {
    fn source() -> T;
}

pub trait SourceCtx<T> {
    type Source: Source<T>;
}

// endregion:   --- Context traits

// region:      --- impls for NullCtx

pub type NullCtx = ();

pub type NullSubCtx = ();

impl LocalizedMsg for NullSubCtx {
    fn localized_msg<'a>(_kind: &'a str, _locale: impl Deref<Target = str>) -> Option<&'a str> {
        None
    }
}

impl Locale for NullSubCtx {
    fn locale() -> impl Deref<Target = str> {
        ""
    }
}

impl LocaleCtx for NullCtx {
    type Locale = NullSubCtx;
}

impl ErrCtx for NullCtx {
    type LocalizedMsg = NullSubCtx;
}

// endregion:   --- impls for NullCtx
