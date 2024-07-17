use serde::{ser::SerializeStructVariant, Serialize};
use std::{error::Error as StdError, fmt::Debug};
use thiserror::Error;

use crate::SerError;

#[derive(Error, Debug)]
pub enum AppError<C> {
    #[error("{0}")]
    Core(C),

    #[error("library error due to: [{0}]")]
    LibraryErrorStr(String),

    #[error("library error due to: [{source}]")]
    LibraryError { source: Box<dyn SerError> },
}

impl<C> AppError<C> {
    /// The `cause`'s `to_string()` value is wrapped in an [`AppError::LibraryErrorStr`]
    pub fn library_error_str<T: StdError>(cause: &T) -> AppError<C> {
        Self::LibraryErrorStr(cause.to_string())
    }

    /// The `source` is wrapped in an [`AppError::LibraryError`]
    pub fn library_error<T: SerError + 'static>(source: T) -> AppError<C> {
        Self::LibraryError {
            source: Box::new(source),
        }
    }
}

impl<C: Serialize> Serialize for AppError<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AppError::Core(core_err) => {
                // serializer.serialize_newtype_variant("AppError", 0, "Core", &core_err.to_json())
                core_err.serialize(serializer)
            }
            AppError::LibraryErrorStr(s) => {
                serializer.serialize_newtype_variant("AppError", 1, "LibraryErrorStr", s)
            }
            AppError::LibraryError { source } => {
                let mut state =
                    serializer.serialize_struct_variant("AppError", 2, "LibraryError", 1)?;
                state.serialize_field("source", &source.to_json())?;
                state.end()
            }
        }
    }
}
