#![allow(clippy::disallowed_names)]

use foa::{
    context::{ErrCtx, Locale, LocalizedMsg},
    string_utils::interpolated_localized_msg,
};
use regex::Regex;
use serde::Serialize;
use std::{fmt::Debug, marker::PhantomData};
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
enum XyzError<CTX>
where
    CTX: ErrCtx,
{
    #[error("{:?}", self)]
    // #[error("user with name \"{0}\" already exists")]
    UsernameDuplicate(String),

    #[error("{:?}", self)]
    UsernameEmpty,

    #[error("{:?}", self)]
    MixedError(i32, &'static str, String),

    #[error("{}", interpolated_error_enum_localization::<CTX>(self))]
    MixedError1(i32, &'static str, String),

    #[error("{}", self)]
    _Unused(PhantomData<CTX>),
}

fn interpolated_error_enum_localization<CTX>(err_item: impl Debug) -> String
where
    CTX: ErrCtx,
{
    let debug_str = format!("{err_item:?}");

    if cfg!(debug_assertions) {
        return debug_str;
    }

    let Some((kind, args)) = parse_tuple_variant(&debug_str) else {
        return "invalid message spec".to_owned();
    };
    let args = args
        .into_iter()
        .map(|arg| arg.to_owned())
        .collect::<Vec<_>>();

    interpolated_localized_msg::<CTX>(kind, &args)
}

#[derive(Debug, Clone)]
struct Ctx0;
struct Ctx0TypeI;

impl LocalizedMsg for Ctx0TypeI {
    fn localized_msg<'a>(kind: &'a str, locale: &'a str) -> Option<&'a str> {
        let res = match locale {
            "en-ca" => match kind {
                "err_kind_0" => "no args",
                "err_kind_1" => "one arg is {} and that's it",
                "err_kind_2" => "two args are {} and {} and that's it",
                "MixedError1" => "example of error enum localization: {}, {}, and {} are the args",
                _ => return None,
            },
            "pt-br" => match kind {
                "err_kind_0" => "nenhum parâmetro",
                "err_kind_1" => "um parâmetro {} e é só",
                "err_kind_2" => "dois parâmetros {} e {} e nada mais",
                _ => return None,
            },
            _ => return None,
        };
        Some(res)
    }
}

impl Locale for Ctx0TypeI {
    fn locale<'a>() -> &'a str {
        "en-ca"
    }
}

impl ErrCtx for Ctx0 {
    type Locale = Ctx0TypeI;
    type LocalizedMsg = Ctx0TypeI;
}

type MyXyzError = XyzError<Ctx0>;

pub fn parse_tuple_variant(debug_str: &str) -> Option<(&str, Vec<&str>)> {
    let all_re =
    // Regex::new(r#"(\w+)(\((((\d+)|(,\s*)|"(\w+)")+)\))?"#).expect("invalid regex code");
    Regex::new(r#"(\w+)(\(([^)]*)\))?"#).expect("invalid regex code");
    let all_caps = all_re.captures(debug_str)?;

    let key = all_caps.get(1)?.as_str();

    let Some(args_match) = all_caps.get(3) else {
        return Some((key, vec![]));
    };
    let args_str = args_match.as_str();
    let args_re = Regex::new(r#"(\d+)|,\s*|"(\w+)""#).expect("invalid regex code");

    // skip the commas
    let args_caps_iter = args_re.captures_iter(args_str).step_by(2);

    let mut args = Vec::new();
    for caps in args_caps_iter {
        let arg = if let Some(cap) = caps.get(1) {
            // number arg
            cap.as_str()
        } else {
            // string arg
            caps.get(2)?.as_str()
        };

        args.push(arg);
    }

    Some((key, args))
}

fn main() {
    {
        let err = MyXyzError::UsernameDuplicate("abc".to_owned());
        println!("{err}");
    }

    {
        let err = MyXyzError::UsernameEmpty;
        println!("{err}");
    }

    {
        let err = MyXyzError::MixedError(42, "xyz", "dkdk".to_owned());
        println!("{err}");

        let debug_str = format!("{err}");
        let parsed_debug = parse_tuple_variant(&debug_str);
        println!("parsed_debug={parsed_debug:?}");
    }

    {
        let err = MyXyzError::MixedError1(42, "xyz", "dkdk".to_owned());
        println!("{err}");
    }
}
