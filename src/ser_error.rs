use serde::Serialize;
use serde_json::Value;
use std::{error::Error as StdError, fmt::Display};

//==============
// Error utils

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
        buf.push_str(", SOURCE=[");
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

//==============
// SerializableError

pub trait SerializableError: StdError {
    fn to_json(&self) -> Value;

    fn src(&self) -> Option<&(dyn StdError + 'static)> {
        StdError::source(self)
    }
}

impl<T> SerializableError for T
where
    T: StdError + Serialize,
{
    fn to_json(&self) -> Value {
        serde_json::to_value(self).expect("serde_json::to_value() error")
    }
}

impl StdError for Box<dyn SerializableError> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.src()
    }
}

//==============
// StdBoxError

#[derive(Debug)]
pub struct StdBoxError(Box<dyn StdError>);

impl StdBoxError {
    pub fn new(inner: impl StdError + 'static) -> Self {
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
        serializer.serialize_str(&error_recursive_msg(self))
    }
}

//==============
// SerBoxError

#[derive(Debug)]
pub struct SerBoxError(Box<dyn SerializableError>);

impl SerBoxError {
    pub fn new(inner: impl SerializableError + 'static) -> Self {
        Self(Box::new(inner))
    }

    pub fn as_dyn_std_error(&self) -> &(dyn StdError + 'static) {
        &self.0 as &dyn StdError
    }
}

impl Display for SerBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for SerBoxError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl Serialize for SerBoxError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_json().serialize(serializer)
    }
}
