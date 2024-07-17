use serde::Serialize;
use serde_json::Value;
use std::{error::Error as StdError, fmt::Display};

trait SerializableError: StdError {
    fn to_json(&self) -> Value;
}

impl<T> SerializableError for T
where
    T: StdError + Serialize,
{
    fn to_json(&self) -> Value {
        serde_json::to_value(self).expect("serde_json::to_value() error")
    }
}

#[derive(Debug)]
pub struct SerError(Box<dyn SerializableError>);

impl SerError {
    pub fn new(source: impl StdError + Serialize + 'static) -> Self {
        Self(Box::new(source))
    }
}

impl Display for SerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for SerError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl Serialize for SerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_json().serialize(serializer)
    }
}
