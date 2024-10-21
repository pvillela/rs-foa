//! Demonstrates serialization/deserialization and protection of sensitive information with errors with a
//! [`Props`] payload.
//! Run the example in both dev and prod (`-r`) to see the hashing of sensitive info in action.

use foa::error::{self, BacktraceSpec, DeserError, FullKind, Tag};
use serde::{Deserialize, Serialize};

static FOO_TAG: Tag = Tag("FOO");

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Pld(String);

/// Used to construct errors without sensitive data.
static FOO_ERROR: FullKind<Pld, 1, false> =
    FullKind::new_with_payload("FOO_ERROR", Some("foo message: {xyz}"), &FOO_TAG)
        .with_prop_names(["xyz"])
        .with_backtrace(BacktraceSpec::Env);

static BAR_TAG: Tag = Tag("BAR");

/// Used to construct errors with sensitive data.
static BAR_ERROR: FullKind<Pld, 2, false> =
    FullKind::new_with_payload("BAR_ERROR", Some("bar message: {abc}, {!email}"), &BAR_TAG)
        .with_prop_names(["abc", "!email"])
        .with_backtrace(BacktraceSpec::Env);

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
    main()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Non-sensitive info examples with `SerError` without payload or source.
    {
        let err = FOO_ERROR
            .error_with_values_and_payload(["hi there!".into()], Pld("foo-payload".into()));
        println!("*** err={err:?}");
        let ser_err = err
            .to_sererror_without_pld_or_src([error::StringSpec::Dbg, error::StringSpec::Recursive]);
        println!("*** ser_err={ser_err:?}");
        let json_string = serde_json::to_string(&ser_err)?;
        println!("*** json_string={json_string:?}");
        let deser_err: DeserError = serde_json::from_str(&json_string)?;
        println!("*** deser_err={deser_err:?}");
        let exp_deser_err = DeserError::from(ser_err);
        println!("*** exp_deser_err={exp_deser_err:?}");

        assert_eq!(exp_deser_err, deser_err, "DserError assertion");
    }

    println!();

    // Non-sensitive info examples with `SerError` with payload.
    {
        let err0 = FOO_ERROR
            .error_with_values_and_payload(["hi there!".into()], Pld("foo-payload".into()));
        let err = err0.downcast_payload::<Pld>()?;
        println!("*** err={err:?}");

        let ser_err = err.into_sererror_with_pld([]);
        println!("*** ser_err={ser_err:?}");
        let json_string = serde_json::to_string(&ser_err)?;
        println!("*** json_string={json_string:?}");
        let deser_err: DeserError<Box<Pld>> = serde_json::from_str(&json_string)?;
        println!("*** deser_err={deser_err:?}");
        let mut exp_deser_err = DeserError::from(ser_err);
        exp_deser_err.props = exp_deser_err.props.safe_props().into();
        println!("*** exp_deser_err={exp_deser_err:?}");

        assert_eq!(exp_deser_err.kind_id, deser_err.kind_id, "kind_id");
        assert_eq!(exp_deser_err.msg, deser_err.msg, "msg");
        assert_eq!(exp_deser_err.tag, deser_err.tag, "tag");
        assert_eq!(exp_deser_err.other, deser_err.other, "other");

        assert_eq!(exp_deser_err.payload, deser_err.payload, "payload");
        assert_eq!(exp_deser_err, deser_err, "DserErrorExt assertion");

        println!("=================== before box deref");
        let pld = deser_err.payload;
        println!("*** pld={pld:?}");
        let exp_pld = exp_deser_err.payload;
        println!("*** exp_pld={exp_pld:?}");
        println!("=================== after box deref");
        assert_eq!(exp_pld, pld);
    }

    println!();

    // Sensitive info examples with `SerError` and payload.
    {
        let err0 = BAR_ERROR.error_with_values_and_payload(
            ["hi there!".into(), "bar@example.com".into()],
            Pld("bar-payload".into()),
        );
        let err = err0.downcast_payload::<Pld>()?;
        println!("*** err={err:?}");

        let ser_err = err.into_sererror_with_pld([]);
        println!("*** ser_err={ser_err:?}");
        let json_string = serde_json::to_string(&ser_err)?;
        println!("*** json_string={json_string:?}");
        let deser_err: DeserError<Box<Pld>> = serde_json::from_str(&json_string)?;
        println!("*** deser_err={deser_err:?}");
        let mut exp_deser_err = DeserError::from(ser_err);
        exp_deser_err.props = exp_deser_err.props.safe_props().into();
        println!("*** exp_deser_err={exp_deser_err:?}");

        assert_eq!(exp_deser_err.kind_id, deser_err.kind_id, "kind_id");
        assert_eq!(exp_deser_err.msg, deser_err.msg, "msg");
        assert_eq!(exp_deser_err.tag, deser_err.tag, "tag");
        assert_eq!(exp_deser_err.other, deser_err.other, "other");

        assert_eq!(exp_deser_err.payload, deser_err.payload, "payload");
        assert_eq!(exp_deser_err, deser_err, "DserErrorExt assertion");

        println!("=================== before box deref");
        let pld = deser_err.payload;
        println!("*** pld={pld:?}");
        let exp_pld = exp_deser_err.payload;
        println!("*** exp_pld={exp_pld:?}");
        println!("=================== after box deref");
        assert_eq!(exp_pld, pld);
    }
    Ok(())
}
