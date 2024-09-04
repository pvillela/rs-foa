use crate::{
    context::LocaleSelf,
    db::sqlx::{AsyncTlTxFn, AsyncTxFn, DbCtx},
    tokio::task_local::{TaskLocal, TaskLocalCtx},
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

pub async fn handler_tx<CTX, F>(Json(input): Json<F::In>) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: DbCtx,
    F: AsyncTxFn<CTX>,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    let output = F::in_tx(input).await?;
    Ok(Json(output))
}

#[derive(Clone)]
pub struct TlHeaders {
    pub headers: HeaderMap,
}

impl LocaleSelf for TlHeaders {
    fn locale(&self) -> &str {
        let header_value = self.headers.get("Accept-Language");
        match header_value {
            None => "en-CA",
            Some(v) => v.to_str().unwrap_or("en-CA"),
        }
    }
}

pub async fn handler_tx_headers<CTX, F, D>(
    headers: HeaderMap,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: DbCtx + TaskLocalCtx<D>,
    CTX::TaskLocal: TaskLocal<D, ValueType = TlHeaders>,
    F: AsyncTlTxFn<CTX, D>,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    let output = F::tl_scoped_in_tx(TlHeaders { headers }, input).await?;
    Ok(Json(output))
}
