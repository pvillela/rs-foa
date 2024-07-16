pub trait LocalizedMsg {
    fn localized_msg<'a>(kind: &'a str, locale: &'a str) -> Option<&'a str>;
}

pub trait Locale {
    fn locale<'a>() -> &'a str;
}

//=============
// impls for NullCtx

#[derive(Debug)]
pub struct NullCtx;

impl LocalizedMsg for NullCtx {
    fn localized_msg<'a>(_kind: &'a str, _locale: &'a str) -> Option<&'a str> {
        None
    }
}

impl Locale for NullCtx {
    fn locale<'a>() -> &'a str {
        ""
    }
}
