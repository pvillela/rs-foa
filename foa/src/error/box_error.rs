use serde::Serialize;
use serde_json::Value;
use std::{any::Any, error::Error as StdError, fmt::Display};

// region:      --- utils

pub fn error_chain(err: &(dyn StdError)) -> Vec<&(dyn StdError)> {
    let mut vec = Vec::new();
    vec.push(err);

    let mut source = err.source();

    while let Some(cause) = source {
        vec.push(cause);
        source = cause.source();
    }

    vec
}

pub fn error_recursive_msg(err: &(dyn StdError)) -> String {
    let chain = error_chain(err);
    let mut chain_iter = chain.iter();
    let mut buf = String::new();

    let first = chain_iter
        .next()
        .expect("error chain always has a first element");
    buf.push_str(&first.to_string());

    for item in chain_iter {
        buf.push_str(", source_msg=[");
        buf.push_str(&item.to_string());
    }

    // Push the appropriate number of closing braces to the result string.
    // It would have been easier and maybe faster to just use a loop.
    let mut bracket = [0; 1];
    ']'.encode_utf8(&mut bracket);
    let closing_vec = vec![bracket[0]; chain.len() - 1];
    let closing_str = String::from_utf8(closing_vec).expect("vec should be utf8 by construction");
    buf.push_str(&closing_str);

    buf
}

// endregion    --- utils

// region:      --- JsonError

pub trait JsonError: StdError + Send + Sync + 'static + Any {
    fn to_json(&self) -> Value;

    fn src(&self) -> Option<&(dyn StdError + 'static)> {
        StdError::source(self)
    }
}

impl<T> JsonError for T
where
    T: StdError + Serialize + Send + Sync + 'static,
{
    fn to_json(&self) -> Value {
        serde_json::to_value(self).expect("serde_json::to_value() error")
    }
}

impl StdError for Box<dyn JsonError> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.src()
    }
}

// endregion:   --- JsonError

// region:      --- StdBoxError

#[derive(Debug)]
pub struct StdBoxError(Box<dyn StdError + Send + Sync + 'static>);

impl StdBoxError {
    pub fn new(inner: impl StdError + Send + Sync + 'static) -> Self {
        Self(Box::new(inner))
    }

    pub fn as_dyn_std_error(&self) -> &(dyn StdError + 'static) {
        self.0.as_ref()
    }
}

impl Display for StdBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for StdBoxError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl Serialize for StdBoxError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut text = String::new();
        text.push_str("recursive_msg(");
        text.push_str(&error_recursive_msg(self));
        text.push(')');
        serializer.serialize_str(&text)
    }
}

// endregion:   --- StdBoxError

// region:      --- JsonBoxError

#[derive(Debug)]
pub struct JsonBoxError(pub Box<dyn JsonError>);

impl JsonBoxError {
    pub fn new(inner: impl JsonError) -> Self {
        Self(Box::new(inner))
    }

    fn as_dyn_std_error(&self) -> &(dyn StdError + 'static) {
        &self.0 as &dyn StdError
    }
}

impl Display for JsonBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for JsonBoxError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl Serialize for JsonBoxError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_json().serialize(serializer)
    }
}

// endregion:   --- JsonBoxError

// region:      --- BoxError

#[derive(Debug)]
#[allow(private_interfaces)]
pub enum BoxError {
    Std(StdBoxError),
    Ser(JsonBoxError),
}

impl BoxError {
    pub fn new_ser(inner: impl JsonError) -> Self {
        Self::Ser(JsonBoxError::new(inner))
    }

    pub fn new_std(inner: impl StdError + Send + Sync + 'static) -> Self {
        Self::Std(StdBoxError::new(inner))
    }

    pub fn as_dyn_std_error(&self) -> &(dyn StdError + 'static) {
        match self {
            Self::Std(err) => err.as_dyn_std_error(),
            Self::Ser(err) => err.as_dyn_std_error(),
        }
    }
}

impl Display for BoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Std(err) => err.fmt(f),
            Self::Ser(err) => err.fmt(f),
        }
    }
}

impl StdError for BoxError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Std(err) => err.source(),
            Self::Ser(err) => err.source(),
        }
    }
}

impl Serialize for BoxError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Std(err) => err.serialize(serializer),
            Self::Ser(err) => err.serialize(serializer),
        }
    }
}

// endregion:   --- BoxError
