use crate::{
    error::{BacktraceSpec, TypedErrorKind, VALIDATION_TAG},
    Error,
};
use valid::ValidationError;

pub const VALIDATION_ERROR: TypedErrorKind<ValidationError> =
    TypedErrorKind::new("VALIDATION_ERROR", BacktraceSpec::No, Some(&VALIDATION_TAG));

impl From<ValidationError> for Error {
    fn from(value: ValidationError) -> Self {
        VALIDATION_ERROR.error(value)
    }
}
