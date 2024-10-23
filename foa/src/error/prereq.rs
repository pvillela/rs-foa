use crate::error::TRUNC;
use crate::hash::hash_sha256_of_str_arr;
use crate::string;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use std::result;

//===========================
// region:      --- NullError

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NullError;

impl Display for NullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl StdError for NullError {}

// endregion:   --- NullError

//===========================
// region:      --- Traits

pub trait SendSyncStaticError: StdError + Send + Sync + 'static {}

impl<T> SendSyncStaticError for T where T: StdError + Send + Sync + 'static + ?Sized {}

// endregion:   --- Traits

//===========================
// region:      --- Tag

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Tag(pub &'static str);

// endregion:   --- Tag

//===========================
// region:      --- BacktraceSpec

/// Specifies different backtrace generation modes.
#[derive(Debug, Clone, Copy)]
pub enum BacktraceSpec {
    /// A backtrace is always generated
    Yes,
    /// A backtrace is never generated
    No,
    /// Backtrace generation is based on environment variables as per
    /// [`std::backtrace::Backtrace`](https://doc.rust-lang.org/std/backtrace/struct.Backtrace.html).
    Env,
}

// endregion:   --- BacktraceSpec

//===========================
// region:      --- KindId

#[derive(Serialize)]
pub struct KindId(pub &'static str);

impl KindId {
    fn address(&self) -> usize {
        self as *const Self as usize
    }
}

impl Debug for KindId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            f.write_fmt(format_args!("KindId({}, {})", self.0, self.address()))
        } else {
            f.write_fmt(format_args!("KindId({})", self.0))
        }
    }
}

impl PartialEq for KindId {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Eq for KindId {}

// endregion:   --- KindId

//===========================
// region:      --- Props

#[derive(Deserialize, Clone, PartialEq, Eq)]
pub struct Props {
    pub(crate) pairs: Vec<(String, String)>,
    pub(crate) protected: bool,
}

impl Props {
    pub fn pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        self.pairs.iter().map(|p| (p.0.as_str(), p.1.as_str()))
    }

    pub fn prop_value(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|&p| p.0 == key)
            .map(|p| p.1.as_str())
    }

    /// Hashes value of fields whose names starts with '!'.
    fn hashed_pairs(&self) -> (Vec<(String, String)>, bool) {
        let mut protected = false;
        let pairs = self
            .pairs
            .iter()
            .map(|(name, value)| {
                let value = if name.starts_with("!") {
                    protected = true;
                    let vhash = hash_sha256_of_str_arr(&[value]);
                    string::base64_encode_trunc_of_u8_arr(&vhash, TRUNC)
                } else {
                    value.to_owned()
                };
                (name.to_owned(), value)
            })
            .collect::<Vec<_>>();
        (pairs, protected)
    }

    pub fn safe_props(&self) -> Self {
        if cfg!(debug_assertions) || self.protected {
            self.clone()
        } else {
            let (pairs, protected) = self.hashed_pairs();
            Self { pairs, protected }
        }
    }
}

impl Debug for Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pairs = if cfg!(debug_assertions) || self.protected {
            &self.pairs
        } else {
            &self.hashed_pairs().0
        };
        f.write_str("Props { pairs: ")?;
        pairs.fmt(f)?;
        f.write_str(", protected: ")?;
        Debug::fmt(&self.protected, f)?;
        f.write_str(" }")
    }
}

impl Serialize for Props {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Props", 2)?;
        if cfg!(debug_assertions) || self.protected {
            state.serialize_field("pairs", &self.pairs)?;
            state.serialize_field("protected", &self.protected)?;
        } else {
            let (pairs, protected) = self.hashed_pairs();
            state.serialize_field("pairs", &pairs)?;
            state.serialize_field("protected", &protected)?;
        };
        state.end()
    }
}

// endregion:   --- Props
