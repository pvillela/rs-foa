//! [`ErrorTag`] instances that are commonly used.

use super::ErrorTag;

pub static INTERNAL_TAG: ErrorTag = ErrorTag("INTERNAL");

pub static RUNTIME_TAG: ErrorTag = ErrorTag("RUNTIME");

pub static VALIDATION_TAG: ErrorTag = ErrorTag("VALIDATION");
