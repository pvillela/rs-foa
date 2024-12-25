//! Comprehensive end-to-end testing of error creation, downcasting, mapping, serialization, deserialization.

use foa::{
    error::{
        BacktraceSpec, BasicKind, FullKind, JserBoxError, KindTypeInfo, PayloadKind, PropsKind,
        StringSpec, Tag, TrivialError,
    },
    Error,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Pld {
    x: u32,
    y: String,
}

const PROP_NAMES: [&'static str; 2] = ["a", "!b"];
const PROP_VALUES: [&'static str; 2] = ["cleartext", "sensitive"];

static BASIC_TAG: Tag = Tag("BASIC");
static BASIC_ERROR: BasicKind = BasicKind::new("BASIC_ERROR", None, &BASIC_TAG);

static PROPS_TAG: Tag = Tag("PROPS");
static PROPS_ERROR: PropsKind<2> =
    BasicKind::new("PROPS_ERROR", None, &PROPS_TAG).with_prop_names(PROP_NAMES);

static PAYLOAD_TAG: Tag = Tag("PAYLOAD");
static PAYLOAD_ERROR: PayloadKind<Pld> =
    PayloadKind::new_with_payload("PAYLOAD_ERROR", None, &PAYLOAD_TAG);

static FULL_TAG: Tag = Tag("FULL");
static FULL_ERROR: FullKind<Pld, 2, TrivialError> = BasicKind::new("FULL_ERROR", None, &FULL_TAG)
    .with_prop_names(PROP_NAMES)
    .with_payload()
    .with_src()
    .with_backtrace(BacktraceSpec::Env)
    .with_ref_id();

fn basic_error() -> Error {
    BASIC_ERROR.error()
}

fn props_error() -> Error {
    PROPS_ERROR.error_with_values(PROP_VALUES)
}

fn payload() -> Pld {
    Pld {
        x: 42,
        y: "the answer".into(),
    }
}

fn payload_error() -> Error {
    PAYLOAD_ERROR.error_with_payload(payload())
}

fn full_error() -> Error {
    let src = TrivialError("trivial");
    FULL_ERROR.error_with_values_payload_src(PROP_VALUES, payload(), src)
}

const STR_SPECS: [StringSpec; 3] = [
    StringSpec::Recursive,
    StringSpec::Backtrace,
    StringSpec::SourceDbg,
];

struct JserError {
    dwncst_none: JserBoxError,
    dwncst_pld: JserBoxError,
    dwncst_src: JserBoxError,
    dwncst_both: JserBoxError,
}

struct JstrError {
    dwncst_none: String,
    dwncst_pld: String,
    dwncst_src: String,
    dwncst_both: String,
}

fn error_into_jser<K: KindTypeInfo>(err: impl Fn() -> Error, _kind: &K) -> JserError
where
    K::Pld: Serialize,
    K::Src: Serialize,
{
    let dwncst_none = err().to_sererror_no_payload_src(STR_SPECS);
    let dwncst_pld = {
        let d_err = err().force_downcast_payload_for_kind(_kind);
        d_err.into_sererror_with_payload(STR_SPECS)
    };
    let dwncst_src = {
        let d_err = err().force_downcast_src_for_kind(_kind);
        d_err.into_sererror_with_src(STR_SPECS)
    };
    let dwncst_both = {
        let d_err = err().force_downcast_payload_src_for_kind(_kind);
        d_err.into_sererror_with_payload_src(STR_SPECS)
    };
    JserError {
        dwncst_none: dwncst_none.into(),
        dwncst_pld: dwncst_pld.into(),
        dwncst_src: dwncst_src.into(),
        dwncst_both: dwncst_both.into(),
    }
}

fn jser_mapper(err: impl Fn() -> Error) -> JserError {
    match err().kind_id() {
        k if k == BASIC_ERROR.kind_id() => error_into_jser(err, &BASIC_ERROR),
        k if k == PROPS_ERROR.kind_id() => error_into_jser(err, &PROPS_ERROR),
        k if k == PAYLOAD_ERROR.kind_id() => error_into_jser(err, &PAYLOAD_ERROR),
        k if k == FULL_ERROR.kind_id() => error_into_jser(err, &FULL_ERROR),
        _ => panic!("some error kind is missing"),
    }
}

fn deser_mapper(err: Error, jstr: JstrError) {}

#[test]
fn test() {
    main();
}

fn main() {
    let errs = [basic_error, props_error, payload_error, full_error];

    for err in errs {
        let jser = jser_mapper(err);
        let jstr = JstrError {
            dwncst_none: serde_json::to_string(&jser.dwncst_none).unwrap(),
            dwncst_pld: serde_json::to_string(&jser.dwncst_pld).unwrap(),
            dwncst_src: serde_json::to_string(&jser.dwncst_src).unwrap(),
            dwncst_both: serde_json::to_string(&jser.dwncst_both).unwrap(),
        };
        deser_mapper(err(), jstr);
    }
}
