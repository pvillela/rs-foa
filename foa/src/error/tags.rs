//! [`ErrorTag`] instances that are commonly used.

use super::ErrorTag;

pub static INTERNAL_ERROR_TAG: ErrorTag = ErrorTag("INTERNAL");

pub static RUNTIME_ERROR_TAG: ErrorTag = ErrorTag("RUNTIME");

pub static VALIDATION_ERROR_TAG: ErrorTag = ErrorTag("VALIDATION");

pub static UNEXPECTED_ERROR_TAG: ErrorTag = ErrorTag("UNEXPECTED");
