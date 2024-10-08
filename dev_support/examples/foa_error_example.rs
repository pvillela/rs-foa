use foa::error::{
    self, BacktraceSpec, BasicKind, Error, PropsKind, Tag, TrivialError, UNEXPECTED_ERROR,
};

static EG_TAG: Tag = Tag("EG");

static ERROR0: PropsKind<0, false> = BasicKind::new(
    "ERROR0",
    Some("error kind with no args"),
    BacktraceSpec::Env,
    &EG_TAG,
);
static ERROR1: PropsKind<1, true> = PropsKind::with_prop_names(
    "ERROR1",
    Some("error kind with '{xyz}' as single arg"),
    ["xyz"],
    BacktraceSpec::Env,
    &EG_TAG,
);
static ERROR2: PropsKind<2, true> = PropsKind::with_prop_names(
    "ERROR2",
    Some("error kind with '{aaa}' and '{bbb}' as args"),
    ["aaa", "bbb"],
    BacktraceSpec::Env,
    &EG_TAG,
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

fn error_unexpected() -> Error {
    UNEXPECTED_ERROR.error(TrivialError("trivial"))
}

fn error_string(err: &Error) -> String {
    err.as_fmt().multi_speced_string([
        error::StringSpec::Dbg,
        error::StringSpec::Decor(
            &error::StringSpec::Recursive,
            Some("recursive_msg=("),
            Some(")"),
        ),
        error::StringSpec::Decor(&error::StringSpec::SourceDbg, Some("source="), None),
        error::StringSpec::Decor(&error::StringSpec::Backtrace, Some("backtrace=\n"), None),
    ])
}

fn print_error(err: &Error) {
    println!("display: {err}");
    println!("debug: {err:?}");
    println!("{}", error_string(&err));
    println!(
        "JSON: {}",
        serde_json::to_string(
            &err.to_sererror([error::StringSpec::Dbg, error::StringSpec::Recursive])
        )
        .unwrap()
    );
}

fn main() {
    println!();

    {
        println!("error0");
        let err = error0();
        print_error(&err);
    }

    println!();

    {
        println!("error1");
        let err = error1();
        print_error(&err);
    }

    println!();

    {
        println!("error2");
        let err = error2();
        print_error(&err);
    }

    println!();

    {
        println!("error_unexpected");
        let err = error_unexpected();
        print_error(&err);
    }
}
