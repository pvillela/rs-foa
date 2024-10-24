use super::{
    static_str::StaticStr, BasicKind, ErrSrcNotTyped, Error, JserBoxError, KindId, KindTypeInfo,
    NullError, Payload, Props, Result, SendSyncStaticError, Tag, LIB_DEPENDENCY_TAG,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt::{Debug, Display},
};

//===========================
// region:      --- SerError

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SerError<PLD, SRC> {
    pub(super) kind_id: &'static KindId,
    pub(super) msg: StaticStr,
    pub(super) tag: &'static Tag,
    pub(super) props: Props,
    pub(super) payload: Option<PLD>,
    pub(super) src: Option<SRC>,
    pub(super) other: BTreeMap<&'static str, String>,
}

impl<PLD, SRC> SerError<PLD, SRC> {
    pub fn kind_id(&self) -> &'static KindId {
        self.kind_id
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn tag(&self) -> &'static Tag {
        self.tag
    }

    pub fn props(&self) -> &Props {
        &self.props
    }

    pub fn payload(&self) -> Option<&PLD> {
        self.payload.as_ref()
    }

    pub fn src(&self) -> Option<&SRC> {
        self.src.as_ref()
    }

    pub fn other(&self) -> &BTreeMap<&'static str, String> {
        &self.other
    }
}

impl<PLD, SRC> Display for SerError<PLD, SRC> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl<PLD: Debug, SRC: StdError + 'static> StdError for SerError<PLD, SRC> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.src {
            Some(src) => Some(src),
            None => None,
        }
    }
}

impl<PLD, SRC> From<SerError<PLD, SRC>> for JserBoxError
where
    PLD: Debug + Send + Sync + 'static + Serialize,
    SRC: StdError + Send + Sync + 'static + Serialize,
{
    fn from(value: SerError<PLD, SRC>) -> Self {
        Self::new(value)
    }
}

// endregion:   --- SerError

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
// region:      --- DeserError

static JSON_DESER_ERROR: BasicKind<ErrSrcNotTyped> =
    BasicKind::new("JSON_DESER_ERROR", None, &LIB_DEPENDENCY_TAG);

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        JSON_DESER_ERROR.error(value)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeserError<PLD = (), SRC = NullError> {
    pub kind_id: DeserKindId,
    pub msg: String,
    pub tag: DeserTag,
    pub props: Props,
    pub payload: Option<PLD>,
    pub src: Option<SRC>,
    pub other: BTreeMap<String, String>,
}

impl DeserError {
    pub fn for_kind<K: KindTypeInfo>(
        _kind: &K,
        json_string: String,
    ) -> Result<DeserError<K::Pld, K::Src>>
    where
        K::Pld: Payload + DeserializeOwned,
        K::Src: SendSyncStaticError + DeserializeOwned,
    {
        serde_json::from_str(&json_string).map_err(|err| err.into())
    }
}

impl<PLD, SRC> Display for DeserError<PLD, SRC> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl<PLD: Debug, SRC: StdError + 'static> StdError for DeserError<PLD, SRC> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.src {
            Some(src) => Some(src),
            None => None,
        }
    }
}

impl<PLD, SRC> From<DeserError<PLD, SRC>> for JserBoxError
where
    PLD: Debug + Send + Sync + 'static + Serialize,
    SRC: StdError + Send + Sync + 'static + Serialize,
{
    fn from(value: DeserError<PLD, SRC>) -> Self {
        Self::new(value)
    }
}

impl<PLD, SRC> From<SerError<PLD, SRC>> for DeserError<PLD, SRC> {
    fn from(value: SerError<PLD, SRC>) -> Self {
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
            src: value.src,
            other,
        }
    }
}

// endregion:   --- DeserError

#[cfg(test)]
mod test {
    // See `dev_support::deser_example`
}
