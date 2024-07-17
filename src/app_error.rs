use serde::Serialize;
use std::{error::Error as StdError, fmt::Debug};
use thiserror::Error;

use crate::SerError;

#[derive(Error, Debug, Serialize)]
pub enum AppError<C> {
    #[error("{0}")]
    Core(C),

    #[error("library error due to: [{0}]")]
    LibraryErrorStr(String),

    #[error("library error due to: [{source}]")]
    LibraryError { source: SerError },
}

impl<C> AppError<C> {
    /// The `cause`'s `to_string()` value is wrapped in an [`AppError::LibraryErrorStr`]
    pub fn library_error_str<T: StdError>(cause: &T) -> AppError<C> {
        Self::LibraryErrorStr(cause.to_string())
    }

    /// The `source` is wrapped in an [`AppError::LibraryError`]
    pub fn library_error(source: impl StdError + Serialize + 'static) -> AppError<C> {
        Self::LibraryError {
            source: SerError::new(source),
        }
    }
}
