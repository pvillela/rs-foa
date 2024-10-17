use axum::extract::{FromRequest, FromRequestParts};
use axum::response::IntoResponse;
use std::future::Future;

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
