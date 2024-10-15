//! Demonstrates serialization/deserialization and protection of sensitive information with errors with a
//! [`Props`] payload.
//! Run the example in both dev and prod (`-r`) to see the hashing of sensitive info in action.

use foa::error::{self, BacktraceSpec, DeserError, DeserErrorExt, Props, PropsKind, Tag};

static FOO_TAG: Tag = Tag("FOO");

/// Used to construct errors with sensitive data.
static FOO_ERROR: PropsKind<1, false> = PropsKind::with_prop_names(
    "FOO_ERROR",
    Some("foo message: {xyz}"),
    ["xyz"],
    BacktraceSpec::Env,
    &FOO_TAG,
);

static BAR_TAG: Tag = Tag("BAR");

/// Used to construct errors without sensitive data.
static BAR_ERROR: PropsKind<1, false, false> = PropsKind::with_prop_names(
    "BAR_ERROR",
    Some("foo message: {xyz}"),
    ["xyz"],
    BacktraceSpec::Env,
    &BAR_TAG,
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sensitive info examples with `SerError`.
    {
        let err = FOO_ERROR.error_with_values(["hi there!".into()]);
        println!("*** err={err:?}");
        let ser_err = err.to_sererror([error::StringSpec::Dbg, error::StringSpec::Recursive]);
        println!("*** ser_err={ser_err:?}");
        let json_string = serde_json::to_string(&ser_err)?;
        println!("*** json_string={json_string:?}");
        let deser_err: DeserError = serde_json::from_str(&json_string)?;
        println!("*** deser_err={deser_err:?}");
        let exp_deser_err = DeserError::from(&ser_err);
        println!("*** exp_deser_err={exp_deser_err:?}");

        assert_eq!(exp_deser_err, deser_err, "DserError assertion");
    }

    println!();

    // Sensitive info examples with `SerErrorExt`.
    {
        let err0 = FOO_ERROR.error_with_values(["hi there!".into()]);
        let err = err0.into_errorext::<Props>()?;
        println!("*** err={err:?}");

        let ser_err = err.into_sererrorext([]);
        println!("*** ser_err={ser_err:?}");
        let json_string = serde_json::to_string(&ser_err)?;
        println!("*** json_string={json_string:?}");
        let deser_err: DeserErrorExt<Props> = serde_json::from_str(&json_string)?;
        println!("*** deser_err={deser_err:?}");
        let mut exp_deser_err = DeserErrorExt::from(ser_err);
        exp_deser_err.payload = exp_deser_err.payload.safe_props().into();
        println!("*** exp_deser_err={exp_deser_err:?}");

        assert_eq!(exp_deser_err.kind_id, deser_err.kind_id, "kind_id");
        assert_eq!(exp_deser_err.msg, deser_err.msg, "msg");
        assert_eq!(exp_deser_err.tag, deser_err.tag, "tag");
        assert_eq!(exp_deser_err.other, deser_err.other, "other");

        assert_eq!(exp_deser_err.payload, deser_err.payload, "payload");
        assert_eq!(exp_deser_err, deser_err, "DserErrorExt assertion");

        println!("=================== before box deref");
        let pld = *deser_err.payload;
        println!("*** pld={pld:?}");
        let exp_pld = *exp_deser_err.payload;
        println!("*** exp_pld={exp_pld:?}");
        println!("=================== after box deref");
        assert_eq!(exp_pld, pld);
    }

    println!();

    // Non-sensitive info examples with `SerErrorExt`.
    {
        let err0 = BAR_ERROR.error_with_values(["hi there!".into()]);
        let err = err0.into_errorext::<Props>()?;
        println!("*** err={err:?}");

        let ser_err = err.into_sererrorext([]);
        println!("*** ser_err={ser_err:?}");
        let json_string = serde_json::to_string(&ser_err)?;
        println!("*** json_string={json_string:?}");
        let deser_err: DeserErrorExt<Props> = serde_json::from_str(&json_string)?;
        println!("*** deser_err={deser_err:?}");
        let mut exp_deser_err = DeserErrorExt::from(ser_err);
        exp_deser_err.payload = exp_deser_err.payload.safe_props().into();
        println!("*** exp_deser_err={exp_deser_err:?}");

        assert_eq!(exp_deser_err.kind_id, deser_err.kind_id, "kind_id");
        assert_eq!(exp_deser_err.msg, deser_err.msg, "msg");
        assert_eq!(exp_deser_err.tag, deser_err.tag, "tag");
        assert_eq!(exp_deser_err.other, deser_err.other, "other");

        assert_eq!(exp_deser_err.payload, deser_err.payload, "payload");
        assert_eq!(exp_deser_err, deser_err, "DserErrorExt assertion");

        println!("=================== before box deref");
        let pld = *deser_err.payload;
        println!("*** pld={pld:?}");
        let exp_pld = *exp_deser_err.payload;
        println!("*** exp_pld={exp_pld:?}");
        println!("=================== after box deref");
        assert_eq!(exp_pld, pld);
    }
    Ok(())
}
