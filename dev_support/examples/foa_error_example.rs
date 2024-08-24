use std::{error::Error, fmt::Debug};

use foa::{
    context::{ErrCtx, Locale, LocalizedMsg},
    error::{ErrorKind, FoaError},
};

const ERROR0: ErrorKind<0, false> = ErrorKind("ERROR0", "error kind with no args");
const ERROR1: ErrorKind<1, true> = ErrorKind("ERROR1", "error kind with '{}' as single arg");
const ERROR2: ErrorKind<2, true> = ErrorKind("ERROR2", "error kind with '{}' and '{}' as args");

#[derive(Debug, Clone)]
struct Ctx0;
struct Ctx0TypeI;

impl Locale for Ctx0TypeI {
    fn locale<'a>() -> &'a str {
        "en-ca"
    }
}

impl LocalizedMsg for Ctx0TypeI {
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

impl ErrCtx for Ctx0 {
    type Locale = Ctx0TypeI;
    type LocalizedMsg = Ctx0TypeI;
}

fn error0<CTX: ErrCtx>() -> FoaError<CTX> {
    FoaError::new(&ERROR0)
}

fn error1_std<CTX: ErrCtx + Send>() -> FoaError<CTX> {
    FoaError::new_with_args_and_cause_std(&ERROR1, [&42.to_string()], error0::<CTX>())
}

fn error1_ser<CTX: ErrCtx + Send>() -> FoaError<CTX> {
    FoaError::new_with_args_and_cause_ser(&ERROR1, [&42.to_string()], error0::<CTX>())
}

fn error2_std<CTX: ErrCtx + Send>() -> FoaError<CTX> {
    FoaError::new_with_args_and_cause_std(
        &ERROR2,
        [&99.to_string(), "2nd arg"],
        error1_std::<CTX>(),
    )
}

fn error2_ser<CTX: ErrCtx + Send>() -> FoaError<CTX> {
    FoaError::new_with_args_and_cause_ser(
        &ERROR2,
        [&99.to_string(), "2nd arg"],
        error1_ser::<CTX>(),
    )
}

fn print_error<CTX: ErrCtx>(err: FoaError<CTX>) {
    println!("display: {err}");
    println!("debug: {err:?}");
    println!("JSON: {}", serde_json::to_string(&err).unwrap());
    println!("source: {:?}", err.source());
}

fn main() {
    println!("============= NullCtx");
    println!();

    {
        println!("error0");
        let err: FoaError<()> = error0();
        print_error(err);
    }

    println!();

    {
        println!("error1_std");
        let err: FoaError<()> = error1_std();
        print_error(err);
    }

    println!();

    {
        println!("error1_ser");
        let err: FoaError<()> = error1_ser();
        print_error(err);
    }

    println!();

    {
        println!("error2_std");
        let err: FoaError<()> = error2_std();
        print_error(err);
    }

    println!();

    {
        println!("error2_ser");
        let err: FoaError<()> = error2_ser();
        print_error(err);
    }

    println!();
    println!("============= Ctx0");
    println!();

    {
        println!("error0");
        let err: FoaError<Ctx0> = error0();
        print_error(err);
    }

    println!();

    {
        println!("error1_std");
        let err: FoaError<Ctx0> = error1_std();
        print_error(err);
    }

    println!();

    {
        println!("error1_ser");
        let err: FoaError<Ctx0> = error1_ser();
        print_error(err);
    }

    println!();

    {
        println!("error2_std");
        let err: FoaError<Ctx0> = error2_std();
        print_error(err);
    }

    println!();

    {
        println!("error2_ser");
        let err: FoaError<Ctx0> = error2_ser();
        print_error(err);
    }
}
