use serde::Serialize;
use std::{error::Error as StdError, fmt::Debug};
use thiserror::Error;

use crate::{SerBoxError, StdBoxError};

/// Simple error struct to be used by applications. It wraps an error enum of type `C` and adds a couple of variants
/// for errors returned from libraries called by the app.
#[derive(Error, Debug, Serialize)]
pub enum AppError<C> {
    #[error("{0}")]
    Core(C),

    #[error("library error due to: [{source}]")]
    LibraryError { source: StdBoxError },

    #[error("library error due to: [{source}]")]
    LibraryErrorSer { source: SerBoxError },
}

impl<C> AppError<C> {
    /// Wraps `core_error` in an [`AppError::Core`]
    pub fn core(core_error: C) -> Self {
        AppError::Core(core_error)
    }

    /// Wraps `source` in an [`AppError::LibraryErrorStr`]
    pub fn library_error(source: impl StdError + 'static) -> AppError<C> {
        Self::LibraryError {
            source: StdBoxError::new(source),
        }
    }

    /// Wraps `source` in an [`AppError::LibraryError`]
    pub fn library_error_ser(source: impl StdError + Serialize + 'static) -> AppError<C> {
        Self::LibraryErrorSer {
            source: SerBoxError::new(source),
        }
    }
}
