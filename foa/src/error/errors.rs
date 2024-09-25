//! [`ErrorKind`] instances that are commonly used directly or as parents of other error kinds.

use super::{ErrorKind, ErrorTag};

pub const INTERNAL_TAG: ErrorTag = ErrorTag("INTERNAL");

pub const INTERNAL_ERROR: ErrorKind<1, true> =
    ErrorKind::new("INTERNAL_ERROR", "internal error {0}", None);

pub const RUNTIME_TAG: ErrorTag = ErrorTag("RUNTIME");

pub const RUNTIME_ERROR: ErrorKind<1, true> =
    ErrorKind::new("RUNTIME_ERROR", "runtime error {0}", None);

pub const VALIDATION_TAG: ErrorTag = ErrorTag("VALIDATION");

pub const VALIDATION_ERROR: ErrorKind<1, true> =
    ErrorKind::new("VALIDATION_ERROR", "validation error", None);
