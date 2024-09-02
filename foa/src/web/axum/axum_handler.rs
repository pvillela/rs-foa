use crate::db::sqlx::{AsyncTlTxFn, AsyncTxFn, Db};
use crate::error::FoaError;
use crate::tokio::task_local::TaskLocalCtx;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::Json;
use futures::Future;

impl<CTX> IntoResponse for FoaError<CTX> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

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

pub async fn handler_tx<CTX, F>(Json(input): Json<F::In>) -> Result<Json<F::Out>, F::E>
where
    CTX: Db,
    F: AsyncTxFn<CTX>,
    F::In: 'static + serde::Deserialize<'static>,
    F::Out: IntoResponse + Send + Sync,
{
    let output = F::in_tx(input).await?;
    Ok(Json(output))
}

pub async fn handler_tx_headers<CTX, F, D>(
    headers: HeaderMap,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, F::E>
where
    CTX: Db + TaskLocalCtx<D, TaskLocalType = HeaderMap>,
    F: AsyncTlTxFn<CTX, D>,
    F::In: 'static + serde::Deserialize<'static>,
    F::Out: IntoResponse + Send + Sync,
{
    let output = F::tl_scoped_in_tx(headers, input).await?;
    Ok(Json(output))
}
