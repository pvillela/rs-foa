use crate::{
    error::{BacktraceSpec, TypedKind, VALIDATION_ERROR_TAG},
    Error,
};
use valid::ValidationError;

pub static VALIDATION_ERROR: TypedKind<ValidationError> = TypedKind::new(
    "VALIDATION_ERROR",
    BacktraceSpec::No,
    Some(&VALIDATION_ERROR_TAG),
);

impl From<ValidationError> for Error {
    fn from(value: ValidationError) -> Self {
        VALIDATION_ERROR.error(value)
    }
}

#[cfg(test)]
mod test {
    use crate::validation::validc::VALIDATION_ERROR;
    use valid::{constraint::Bound, Validate};

    #[test]
    fn test() {
        let age_delta: i32 = -10;
        let payload = age_delta
            .validate(
                "age_delta must be nonnegative",
                &Bound::ClosedRange(0, i32::MAX),
            )
            .result()
            .expect_err("validation designed to fail");

        let err = VALIDATION_ERROR.error(payload);
        assert!(err.has_kind(VALIDATION_ERROR.kind_id()));
    }
}