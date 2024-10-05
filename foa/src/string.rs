use crate::context::{ErrCtx, Locale, LocalizedMsg};
use base64ct::{Base64, Encoding};

/// Interpolates a string with a list of arguments.
pub fn interpolated_vec<S>(mut raw_msg: &str, args: &[S]) -> String
where
    S: AsRef<str>,
{
    let mut msg = String::with_capacity(raw_msg.len() * 2);
    for arg in args {
        let Some(idx) = raw_msg.find("{}") else {
            return "more error message args than template placeholders".to_owned();
        };
        let prefix = &raw_msg[0..idx];
        msg.push_str(prefix);
        msg.push_str(arg.as_ref());
        raw_msg = &raw_msg[idx + 2..];
    }

    if let Some(_idx) = raw_msg.find("{}") {
        return "fewer error message args than template placeholders".to_owned();
    }

    // push end of `raw_msg`
    msg.push_str(raw_msg);

    msg
}

/// Interpolates a localized message with a list of arguments.
pub fn interpolated_localized_vec<CTX, S>(kind: &str, args: &[S]) -> String
where
    CTX: ErrCtx,
    S: AsRef<str>,
{
    let Some(raw_msg) = localized::<CTX>(kind) else {
        return "invalid message key".to_owned();
    };
    interpolated_vec(raw_msg, args)
}

/// Interpolates a string with properties (list of name-value pairs).
pub fn interpolated_props<'a, P, S1, S2>(raw_msg: &'a str, props: P) -> String
where
    S1: AsRef<str>,
    S2: AsRef<str>,
    P: Iterator<Item = (S1, S2)>,
{
    let mut msg = raw_msg.to_owned();
    for (name, value) in props {
        let name = name.as_ref();
        let value = value.as_ref();
        let placeholder = format!("{{{name}}}");
        msg = msg.replace(&placeholder, &value);
    }
    msg
}

/// Lazily interpolates a string with properties (list of name-value pairs),
/// where the values are returned by functions from a common input.
pub fn interpolated_props_lazy<'a, P, S1, S2, T>(raw_msg: &'a str, props: P, input: &T) -> String
where
    S1: AsRef<str>,
    S2: AsRef<str>,
    P: Iterator<Item = (S1, fn(&T) -> S2)>,
{
    let mut msg = raw_msg.to_owned();
    for (name, value) in props {
        let name = name.as_ref();
        let value = value(input);
        let value = value.as_ref();
        let placeholder = format!("{{{name}}}");
        msg = msg.replace(&placeholder, &value);
    }
    msg
}

/// Interpolates a localized message with properties (list of name-value pairs).
pub fn interpolated_localized_props<'a, CTX, P, S1, S2>(kind: &str, props: P) -> String
where
    CTX: ErrCtx,
    S1: AsRef<str>,
    S2: AsRef<str>,
    P: Iterator<Item = (S1, S2)>,
{
    let Some(raw_msg) = localized::<CTX>(kind) else {
        return "invalid message key".to_owned();
    };
    interpolated_props(raw_msg, props)
}

pub fn localized<CTX>(kind: &str) -> Option<&str>
where
    CTX: ErrCtx,
{
    CTX::LocalizedMsg::localized_msg(kind, CTX::Locale::locale())
}

/// Encodes a byte array as a lower hex string.
pub fn hex_lower_of_u8_arr(arr: &[u8]) -> String {
    arr.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}

/// Encodes a byte array as a Base64 string, truncating it to a given max size.
/// If the max size is greater than or equal to the length of the array then
/// the encoding is returned without truncation.
pub fn base64_encode_trunc_of_u8_arr(arr: &[u8], max_size: usize) -> String {
    let trunc = arr.len().min(max_size);
    Base64::encode_string(&arr[0..trunc])
}

/// Decorates a string with with optional characters around it (usually brackets) and an
/// optional prefix.
pub fn decorated(txt: &str, pre: Option<&str>, post: Option<&str>) -> String {
    let body = match post {
        Some(post) => txt.to_owned() + post,
        None => txt.to_owned(),
    };
    match pre {
        Some(pre) => pre.to_owned() + &body,
        None => body,
    }
}
