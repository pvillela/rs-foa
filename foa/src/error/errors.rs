//! [`ErrorKind`] instances that are commonly used directly or as parents of other error kinds.

use super::{SimpleErrorKind, ErrorTag};

pub const INTERNAL_TAG: ErrorTag = ErrorTag("INTERNAL");

pub const INTERNAL_ERROR: SimpleErrorKind<1, true> =
    SimpleErrorKind::new("INTERNAL_ERROR", "internal error {0}", None);

pub const RUNTIME_TAG: ErrorTag = ErrorTag("RUNTIME");

pub const RUNTIME_ERROR: SimpleErrorKind<1, true> =
    SimpleErrorKind::new("RUNTIME_ERROR", "runtime error {0}", None);

pub const VALIDATION_TAG: ErrorTag = ErrorTag("VALIDATION");

pub const VALIDATION_ERROR: SimpleErrorKind<1, true> =
    SimpleErrorKind::new("VALIDATION_ERROR", "validation error", None);
