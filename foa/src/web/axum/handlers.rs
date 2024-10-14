use crate::error::{self, Error, JserBoxError, VALIDATION_TAG};
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use log::{log, Level};
use std::future::Future;
use valid::ValidationError;

/// Checks a closure for compliance with Axum Handler impl requirements.
pub fn _axum_handler_type_checker_2_args_generic<In1, In2, Out, Fut, S>(
    _f: &(impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static),
) where
    Fut: Future<Output = Out> + Send,
    In1: FromRequestParts<S> + Send,
    In2: FromRequest<S> + Send,
    Out: IntoResponse,
    S: Send + Sync + 'static,
{
}

fn error_string_error_level(err: &Error) -> String {
    err.as_fmt().multi_speced_string([
        error::StringSpec::Dbg,
        error::StringSpec::Decor(
            &error::StringSpec::Recursive,
            Some("recursive_msg=("),
            Some(")"),
        ),
        error::StringSpec::Decor(&error::StringSpec::SourceDbg, Some("source="), None),
        error::StringSpec::Decor(&error::StringSpec::Backtrace, Some("backtrace=\n"), None),
    ])
}

pub fn default_mapper(err: Error) -> (StatusCode, JserBoxError) {
    match err.tag() {
        tag if tag == &VALIDATION_TAG => {
            let status_code = StatusCode::BAD_REQUEST;
            let err_exp_res = err.into_errorext::<ValidationError>();
            match err_exp_res {
                Ok(ee) => (status_code, ee.into_sererrorext([]).into()),
                Err(e) => (
                    status_code,
                    e.to_sererror([error::StringSpec::Dbg, error::StringSpec::Recursive])
                        .into(),
                ),
            }
        }
        _ => {
            log!(Level::Error, "{}", error_string_error_level(&err));
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                err.to_sererror([error::StringSpec::Dbg, error::StringSpec::Recursive])
                    .into(),
            )
        }
    }
}
