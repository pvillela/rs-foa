use crate::{
    context::LocaleSelf,
    db::sqlx::{in_tx, invoke_in_tx, AsyncTxFn, DbCtx},
    fun::AsyncRFn,
    tokio::task_local::{invoke_tl_scoped, TaskLocal, TaskLocalCtx},
    trait_utils::Make,
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

pub async fn handler<F, MF>(Json(input): Json<F::In>) -> Result<Json<F::Out>, Json<F::E>>
where
    F: AsyncRFn,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    MF: Make<F>,
{
    let output = MF::make().invoke(input).await?;
    Ok(Json(output))
}

pub async fn handler_tx<CTX, F, MF>(Json(input): Json<F::In>) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: DbCtx + Sync,
    F: AsyncTxFn<CTX> + Sync,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    MF: Make<F>,
{
    let output = invoke_in_tx(MF::make(), input).await?;
    Ok(Json(output))
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

pub async fn handler_headers<CTX, F, MF, D>(
    headers: HeaderMap,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: TaskLocalCtx<D> + Sync,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncRFn + Sync,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync,
    MF: Make<F>,
{
    let output = invoke_tl_scoped::<CTX, F, D>(MF::make(), (headers, input)).await?;
    Ok(Json(output))
}

pub async fn handler_tx_headers<CTX, F, MF, D>(
    headers: HeaderMap,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: DbCtx + TaskLocalCtx<D> + Sync,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncTxFn<CTX> + Sync,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync,
    MF: Make<F>,
{
    let f_in_tx = in_tx(MF::make()).await;
    let output = invoke_tl_scoped::<CTX, _, _>(f_in_tx, (headers, input)).await?;
    Ok(Json(output))
}
