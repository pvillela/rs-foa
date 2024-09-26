//! [`ErrorTag`] instances that are commonly used.

use super::ErrorTag;

pub const INTERNAL_TAG: ErrorTag = ErrorTag("INTERNAL");

pub const RUNTIME_TAG: ErrorTag = ErrorTag("RUNTIME");

pub const VALIDATION_TAG: ErrorTag = ErrorTag("VALIDATION");
