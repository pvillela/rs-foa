use std::error::Error as _;

use foa::error::{Error, ErrorKind};

const ERROR0: ErrorKind<0, false> = ErrorKind::new("ERROR0", "error kind with no args", [], None);
const ERROR1: ErrorKind<1, true> = ErrorKind::new(
    "ERROR1",
    "error kind with '{xyz}}' as single arg",
    ["xyz"],
    None,
);
const ERROR2: ErrorKind<2, true> = ErrorKind::new(
    "ERROR2",
    "error kind with '{aaa}' and '{bbb}' as args",
    ["aaa", "bbb"],
    None,
);

fn error0() -> Error {
    ERROR0.new_error()
}

fn error1_std() -> Error {
    ERROR1.new_error_with_args([&42.to_string()], error0())
}

fn error1_ser() -> Error {
    ERROR1.new_error_with_args_ser([&42.to_string()], error0())
}

fn error2_std() -> Error {
    ERROR2.new_error_with_args([&99.to_string(), "2nd arg"], error1_std())
}

fn error2_ser() -> Error {
    ERROR2.new_error_with_args_ser([&99.to_string(), "2nd arg"], error1_std())
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
        let err = error1_std();
        print_error(err);
    }

    println!();

    {
        println!("error1_ser");
        let err = error1_ser();
        print_error(err);
    }

    println!();

    {
        println!("error2_std");
        let err = error2_std();
        print_error(err);
    }

    println!();

    {
        println!("error2_ser");
        let err = error2_ser();
        print_error(err);
    }
}
