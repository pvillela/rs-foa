use serde::Deserialize;
use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt::{Debug, Display},
};

use super::Payload;

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
// region:      --- DeserError, DeserErrorExp

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct DeserErrorExp<T: Payload> {
    pub kind_id: DeserKindId,
    pub msg: String,
    pub tag: DeserTag,
    pub payload: T,
    pub other: BTreeMap<String, String>,
}

impl<T: Payload> Display for DeserErrorExp<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl<T: Payload> StdError for DeserErrorExp<T> {}

// endregion:   --- DeserError, DeserErrorExp

#[cfg(test)]
mod test {
    use crate::error::{BacktraceSpec, Props, PropsKind, Tag};

    static FOO_TAG: Tag = Tag("FOO");

    static FOO_ERROR: PropsKind<1, false> = PropsKind::with_prop_names(
        "FOO_ERROR",
        Some("foo message: {xyz}"),
        ["xyz"],
        BacktraceSpec::Env,
        &FOO_TAG,
    );

    #[test]
    fn test_into_errorexp_props() {
        let err = FOO_ERROR.error_with_values(["hi there!".into()]);

        assert!(err.has_kind(FOO_ERROR.kind_id()));
        assert_eq!(err.to_string(), "foo message: {xyz}");

        let res = err.into_errorexp::<Props>();
        match res {
            Ok(ee) => assert_eq!(ee.kind_id(), FOO_ERROR.kind_id()),
            Err(_) => unreachable!(),
        };
    }
}
