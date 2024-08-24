use serde::Serialize;
use std::{error::Error as StdError, fmt::Debug};
use thiserror::Error;

use super::BoxError;

/// Simple error struct to be used by applications. It wraps an error enum of type `C` and adds a couple of variants
/// for errors returned from libraries called by the app. `C` must be [`Serialize`], which is not a problem for
/// simple error enums.
#[derive(Error, Debug, Serialize)]
pub enum AppError<C> {
    #[error("{0}")]
    Core(C),

    #[error("library error due to: [{source}]")]
    LibraryError { source: BoxError },
}

impl<C> AppError<C> {
    /// Wraps `core_error` in an [`AppError::Core`]
    pub fn core(core_error: C) -> Self {
        AppError::Core(core_error)
    }

    /// Wraps a JSON-serializable `source` in an [`AppError::LibraryError`]. The result is serializable,
    /// taking into account the JSON structure of the source error.
    pub fn library_error_ser(
        source: impl StdError + Serialize + Send + Sync + 'static,
    ) -> AppError<C> {
        Self::LibraryError {
            source: BoxError::new_ser(source),
        }
    }

    /// Wraps a `source` in an [`AppError::LibraryError`]. The result is serializable but the serialization
    /// is unstructured, based on a message string.
    pub fn library_error_std(source: impl StdError + Send + Sync + 'static) -> AppError<C> {
        Self::LibraryError {
            source: BoxError::new_std(source),
        }
    }
}
