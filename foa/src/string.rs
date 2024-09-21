use crate::context::{ErrCtx, Locale, LocalizedMsg};

pub fn interpolated_string<S>(mut raw_msg: &str, args: &[S]) -> String
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

pub fn interpolated_localized_msg<CTX, S>(kind: &str, args: &[S]) -> String
where
    CTX: ErrCtx,
    S: AsRef<str>,
{
    let Some(raw_msg) = localized_msg::<CTX>(kind) else {
        return "invalid message key".to_owned();
    };
    interpolated_string(raw_msg, args)
}

pub fn localized_msg<CTX>(kind: &str) -> Option<&str>
where
    CTX: ErrCtx,
{
    CTX::LocalizedMsg::localized_msg(kind, CTX::Locale::locale())
}
