use crate::error::FoaError;
use axum::response::IntoResponse;
use axum::{extract::Extension, Json};
use futures::Future;

pub fn handler_of<S, T, Fut>(
    f: impl Fn(S) -> Fut + 'static + Send + Sync + Clone,
) -> impl Fn(Json<S>) -> Fut + Send + Sync + 'static + Clone
where
    S: 'static + serde::Deserialize<'static>,
    T: IntoResponse + Send + Sync,
    Fut: 'static + Future<Output = T> + Send + Sync,
{
    move |Json(input)| f(input)
}

pub fn handler_of2<S1, S2, T, Fut>(
    f: impl Fn(S1, Extension<S2>) -> Fut + 'static + Send + Sync + Clone,
) -> impl Fn(Json<S1>, Extension<S2>) -> Fut + Send + Sync + 'static + Clone
where
    S1: 'static + serde::Deserialize<'static>,
    T: IntoResponse + Send + Sync,
    Fut: 'static + Future<Output = T> + Send + Sync,
{
    move |Json(input), ext| f(input, ext)
}

impl<CTX> IntoResponse for FoaError<CTX> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}
