use std::error::Error;

use foa::{ErrorKind, FoaError, Locale, LocalizedMsg};

const ERROR0: ErrorKind<0, false> = ErrorKind("ERROR0", "error kind with no args");
const ERROR1: ErrorKind<1, false> = ErrorKind("ERROR1", "error kind with '{}' as single arg");
const ERROR2: ErrorKind<2, true> = ErrorKind("ERROR2", "error kind with '{}' and '{}' as args");

#[derive(Debug, Clone)]
struct Ctx0;

impl LocalizedMsg for Ctx0 {
    fn localized_msg<'a>(kind: &'a str, locale: &'a str) -> Option<&'a str> {
        let res = match locale {
            "en-ca" => match kind {
                "ERROR0" => "no args",
                "ERROR1" => "one arg is '{}' and that's it",
                "ERROR2" => "two args are '{}' and '{}' and that's it",
                _ => return None,
            },
            _ => return None,
        };
        Some(res)
    }
}

impl Locale for Ctx0 {
    fn locale<'a>() -> &'a str {
        "en-ca"
    }
}

fn main() {
    println!("============= NullCtx");
    println!();

    {
        let kind = &ERROR0;
        println!("kind: {kind:?}");
        let err: FoaError<()> = FoaError::new(kind);
        println!("display: {err}");
        println!("debug: {err:?}");
    }

    println!();

    {
        let kind = &ERROR1;
        println!("kind: {kind:?}");
        let err: FoaError<()> = FoaError::new_with_args(kind, [&42.to_string()]);
        println!("display: {err}");
        println!("debug: {err:?}");
    }

    println!();

    {
        let kind = &ERROR2;
        println!("kind: {kind:?}");
        let cause: FoaError<()> = FoaError::new_with_args(&ERROR1, [&42.to_string()]);
        let err: FoaError<()> =
            FoaError::new_with_args_and_cause(kind, [&99.to_string(), "2nd arg"], cause);
        println!("display: {err}");
        println!("debug: {err:?}");
        println!("source: {:?}", err.source());
    }

    println!();
    println!("============= Ctx0");
    println!();

    {
        let kind = &ERROR0;
        println!("kind: {kind:?}");
        let err: FoaError<Ctx0> = FoaError::new(kind);
        println!("display: {err}");
        println!("debug: {err:?}");
    }

    println!();

    {
        let kind = &ERROR1;
        println!("kind: {kind:?}");
        let err: FoaError<Ctx0> = FoaError::new_with_args(kind, [&42.to_string()]);
        println!("display: {err}");
        println!("debug: {err:?}");
    }

    println!();

    {
        let kind = &ERROR2;
        println!("kind: {kind:?}");
        let cause: FoaError<Ctx0> = FoaError::new_with_args(&ERROR1, [&42.to_string()]);
        let err: FoaError<Ctx0> =
            FoaError::new_with_args_and_cause(kind, [&99.to_string(), "2nd arg"], cause);
        println!("display: {err}");
        println!("debug: {err:?}");
        println!("source: {:?}", err.source());
    }
}
