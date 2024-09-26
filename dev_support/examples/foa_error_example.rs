use std::error::Error as _;

use foa::error::{Error, PropsErrorKind};

const ERROR0: PropsErrorKind<0, false> =
    PropsErrorKind::new("ERROR0", "error kind with no args", [], None);
const ERROR1: PropsErrorKind<1, true> = PropsErrorKind::new(
    "ERROR1",
    "error kind with '{xyz}' as single arg",
    ["xyz"],
    None,
);
const ERROR2: PropsErrorKind<2, true> = PropsErrorKind::new(
    "ERROR2",
    "error kind with '{aaa}' and '{bbb}' as args",
    ["aaa", "bbb"],
    None,
);

fn error0() -> Error {
    ERROR0.new_error()
}

fn error1() -> Error {
    ERROR1.new_error_with_args([&42.to_string()], error0())
}

fn error2() -> Error {
    ERROR2.new_error_with_args([&99.to_string(), "2nd arg"], error1())
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
