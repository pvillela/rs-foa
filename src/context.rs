pub trait LocalizedMsg {
    fn localized_msg<'a>(kind: &'a str, locale: &'a str) -> Option<&'a str>;
}

pub trait Locale {
    fn locale<'a>() -> &'a str;
}
