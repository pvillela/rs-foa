use foa::{
    error::{BacktraceSpec, BasicKind, PropsKind, Tag},
    Error,
};

static FOO_TAG: Tag = Tag("FOO");

static FOO_ERROR: PropsKind<1, false> = PropsKind::with_prop_names(
    "FOO_ERROR",
    Some("foo message: {xyz}"),
    ["xyz"],
    BacktraceSpec::Yes,
    &FOO_TAG,
);

static BAR_ERROR: BasicKind<true> = BasicKind::new(
    "BAR_ERROR",
    Some("bar message"),
    BacktraceSpec::Env,
    &FOO_TAG,
);

fn out_formatted_string(err: &Error) -> String {
    let mut fmt_spec = "{dbg_string}".to_owned();
    fmt_spec.push_str(", recursive_msg=({recursive_msg})");
    fmt_spec.push_str(", source={source_dbg_string}");
    fmt_spec.push_str(", backtrace=\n{backtrace_string}");
    err.as_fmt().formatted_string(&fmt_spec)
}

fn main() {
    let src = FOO_ERROR.error_with_values(["42"]);
    let err = BAR_ERROR.error(src);
    let out_string = out_formatted_string(&err);
    println!("{out_string}");
}