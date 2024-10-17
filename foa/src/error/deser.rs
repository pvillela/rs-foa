use super::{Payload, SerError, SerErrorExt};
use crate::error::foa_error::Props;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt::{Debug, Display},
};
//===========================
// region:      --- DeserTag

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeserTag(pub String);

// endregion:   --- DeserTag

//===========================
// region:      --- DeserKindId

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeserKindId(pub String);

// endregion:   --- DeserKindId

//===========================
// region:      --- DeserError, DeserErrorExt

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DeserError {
    pub kind_id: DeserKindId,
    pub msg: String,
    pub tag: DeserTag,
    pub props: Props,
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
            props: value.props().clone(),
            tag: DeserTag(value.tag().0.to_owned()),
            other,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeserErrorExt<T: Payload> {
    pub kind_id: DeserKindId,
    pub msg: String,
    pub tag: DeserTag,
    pub props: Props,
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
            props: value.props,
            payload: value.payload,
            other,
        }
    }
}

// endregion:   --- DeserError, DeserErrorExt

#[cfg(test)]
mod test {
    // See `dev_support::deser_example`
}
