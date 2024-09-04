use crate::{
    context::LocaleSelf,
    db::sqlx::{AsyncTxFn, DbCtx, InTx},
    fun::AsyncRFn,
    tokio::task_local::{TaskLocal, TaskLocalCtx, TlScoped},
};
use axum::{http::HeaderMap, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::future::Future;

pub fn handler_of<S, T, Fut>(
    f: impl Fn(S) -> Fut + 'static + Send + Sync + Clone,
) -> impl Fn(Json<S>) -> Fut + Send + Sync + 'static + Clone
where
    S: Deserialize<'static> + 'static,
    T: IntoResponse + Send + Sync,
    Fut: 'static + Future<Output = T> + Send + Sync,
{
    move |Json(input)| f(input)
}

pub async fn handler<F>(Json(input): Json<F::In>) -> Result<Json<F::Out>, Json<F::E>>
where
    F: AsyncRFn,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    let output = F::invoke(input).await?;
    Ok(Json(output))
}

pub async fn handler_tx<CTX, F>(json: Json<F::In>) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: DbCtx,
    F: AsyncTxFn<CTX>,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    handler::<InTx<CTX, F>>(json).await
}

impl LocaleSelf for HeaderMap {
    fn locale(&self) -> &str {
        let header_value = self.get("Accept-Language");
        match header_value {
            None => "en-CA",
            Some(v) => v.to_str().unwrap_or("en-CA"),
        }
    }
}

pub async fn handler_headers<CTX, F, D>(
    headers: HeaderMap,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: TaskLocalCtx<D>,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncRFn,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    let output = TlScoped::<CTX, F, D>::invoke((headers, input)).await?;
    Ok(Json(output))
}

pub async fn handler_tx_headers<CTX, F, D>(
    headers: HeaderMap,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: DbCtx + TaskLocalCtx<D>,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncTxFn<CTX>,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    let output = TlScoped::<CTX, InTx<CTX, F>, D>::invoke((headers, input)).await?;
    Ok(Json(output))
}
