use super::{Payload, SerError, SerErrorExt};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt::{Debug, Display},
};

//===========================
// region:      --- DeserTag

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DeserTag(pub String);

// endregion:   --- DeserTag

//===========================
// region:      --- DeserKindId

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DeserKindId(pub String);

// endregion:   --- DeserKindId

//===========================
// region:      --- DeserError, DeserErrorExt

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DeserError {
    pub kind_id: DeserKindId,
    pub msg: String,
    pub tag: DeserTag,
    pub other: BTreeMap<String, String>,
}

impl Display for DeserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl StdError for DeserError {}

impl From<&SerError> for DeserError {
    fn from(value: &SerError) -> Self {
        let other: BTreeMap<String, String> = value
            .other()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Self {
            kind_id: DeserKindId(value.kind_id().0.to_owned()),
            msg: value.msg().to_owned(),
            tag: DeserTag(value.tag().0.to_owned()),
            other,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DeserErrorExt<T: Payload> {
    pub kind_id: DeserKindId,
    pub msg: String,
    pub tag: DeserTag,
    pub payload: Box<T>,
    pub other: BTreeMap<String, String>,
}

impl<T: Payload> Display for DeserErrorExt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl<T: Payload> StdError for DeserErrorExt<T> {}

impl<T: Payload> From<SerErrorExt<T>> for DeserErrorExt<T> {
    fn from(value: SerErrorExt<T>) -> Self {
        let other: BTreeMap<String, String> = value
            .other()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Self {
            kind_id: DeserKindId(value.kind_id().0.to_owned()),
            msg: value.msg().to_owned(),
            tag: DeserTag(value.tag().0.to_owned()),
            payload: value.payload,
            other,
        }
    }
}

// endregion:   --- DeserError, DeserErrorExt

#[cfg(test)]
mod test {
    use crate::error::{self, BacktraceSpec, DeserError, DeserErrorExt, Props, PropsKind, Tag};

    static FOO_TAG: Tag = Tag("FOO");

    static FOO_ERROR: PropsKind<1, false> = PropsKind::with_prop_names(
        "FOO_ERROR",
        Some("foo message: {xyz}"),
        ["xyz"],
        BacktraceSpec::Env,
        &FOO_TAG,
    );

    #[test]
    fn test_deser() -> Result<(), Box<dyn std::error::Error>> {
        {
            let err = FOO_ERROR.error_with_values(["hi there!".into()]);
            let ser_err = err.to_sererror([error::StringSpec::Dbg, error::StringSpec::Recursive]);
            let json_err = serde_json::to_string(&ser_err)?;
            let deser_err: DeserError = serde_json::from_str(&json_err)?;
            let exp_deser_err = DeserError::from(&ser_err);

            assert_eq!(exp_deser_err, deser_err);
        }

        {
            let err0 = FOO_ERROR.error_with_values(["hi there!".into()]);
            let err = err0.into_errorext::<Props>()?;

            let ser_err =
                err.into_sererrorext([error::StringSpec::Dbg, error::StringSpec::Recursive]);
            let json_err = serde_json::to_string(&ser_err)?;
            let deser_err: DeserErrorExt<Props> = serde_json::from_str(&json_err)?;
            let exp_deser_err = DeserErrorExt::from(ser_err);

            assert_eq!(exp_deser_err, deser_err);
        }
        Ok(())
    }
}
