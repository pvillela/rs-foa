use std::error::Error as _;

use foa::error::{BacktraceSpec, BasicErrorKind, Error, PropsErrorKind};

static ERROR0: PropsErrorKind<0, false> = BasicErrorKind::new(
    "ERROR0",
    Some("error kind with no args"),
    BacktraceSpec::Env,
    None,
);
static ERROR1: PropsErrorKind<1, true> = PropsErrorKind::with_prop_names(
    "ERROR1",
    Some("error kind with '{xyz}' as single arg"),
    ["xyz"],
    BacktraceSpec::Env,
    None,
);
static ERROR2: PropsErrorKind<2, true> = PropsErrorKind::with_prop_names(
    "ERROR2",
    Some("error kind with '{aaa}' and '{bbb}' as args"),
    ["aaa", "bbb"],
    BacktraceSpec::Env,
    None,
);

fn error0() -> Error {
    ERROR0.error()
}

fn error1() -> Error {
    ERROR1.error_with_values([&42.to_string()], error0())
}

fn error2() -> Error {
    ERROR2.error_with_values([&99.to_string(), "2nd arg"], error1())
}

fn print_error(err: Error) {
    println!("display: {err}");
    println!("debug: {err:?}");
    println!("JSON: {}", serde_json::to_string(&err).unwrap());
    println!("source: {:?}", err.source());
}

fn main() {
    println!();

    {
        println!("error0");
        let err = error0();
        print_error(err);
    }

    println!();

    {
        println!("error1_std");
        let err = error1();
        print_error(err);
    }

    println!();

    println!();

    {
        println!("error2_std");
        let err = error2();
        print_error(err);
    }
}
