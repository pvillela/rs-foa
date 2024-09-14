use super::handler_asyncfn2r;
use crate::db::sqlx::{in_tx, AsyncTxFn};
use crate::fun::async_rfn::{AsyncRFn, AsyncRFn2};
use crate::tokio::task_local::{invoke_tl_scoped, TaskLocal};
use crate::trait_utils::Make;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

//=================
// Handlers for AsyncRFn[x]

pub fn handler_asyncrfn<F>(
    f: F,
) -> impl Fn(
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncRFn + Send + Sync + Clone + 'static,
    F::In: DeserializeOwned,
    F::Out: Serialize,
    F::E: Serialize,
{
    move |Json(input)| {
        let f = f.clone();
        Box::pin(async move {
            let output = f.invoke(input).await?;
            Ok(Json(output))
        })
    }
}

pub fn handler_asyncrfn_arc<F>(
    f: F,
) -> impl Fn(
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncRFn + Send + Sync + 'static,
    F::In: DeserializeOwned,
    F::Out: Serialize,
    F::E: Serialize,
{
    handler_asyncrfn(Arc::new(f))
}

pub fn handler_asyncrfn2<F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<
    Box<(dyn Future<Output = (StatusCode, Json<Result<F::Out, F::E>>)> + Send + 'static)>,
>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncRFn2 + Send + Sync + Clone + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    F::Out: Serialize,
    F::E: Serialize,
    S: Send + Sync + 'static,
{
    // move |req_part, Json(input)| {
    //     let f = f.clone();
    //     Box::pin(async move {
    //         let out = f.invoke(req_part, input).await;
    //         let status = match out {
    //             Ok(_) => StatusCode::OK,
    //             Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    //         };
    //         (status, Json(out))
    //     })
    // }
    handler_asyncfn2r(f.into_asyncfn2_when_clone())
}

pub fn handler_asyncrfn2_arc<F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<
    Box<(dyn Future<Output = (StatusCode, Json<Result<F::Out, F::E>>)> + Send + 'static)>,
>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncRFn2 + Send + Sync + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    F::Out: Serialize,
    F::E: Serialize,
    S: Send + Sync + 'static,
{
    handler_asyncrfn2(Arc::new(f))
}

#[deprecated]
pub async fn handler_tx_headers_old<F, MF, TL>(
    parts: Parts,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    TL: TaskLocal<Value = Parts> + Sync + Send + 'static,
    F: AsyncTxFn + Sync + Send + 'static,
    F::In: DeserializeOwned,
    F::Out: Serialize,
    F::E: Serialize,
    MF: Make<F> + 'static,
{
    let f = MF::make();
    let f_in_tx = in_tx(&f);
    let output = invoke_tl_scoped::<_, TL>(&f_in_tx, parts, input).await?;
    Ok(Json(output))
}

#[cfg(test)]
#[allow(deprecated)]
fn _typecheck_handler_tx_headers_old<F, MF, TL>()
where
    TL: TaskLocal<Value = Parts> + Sync + Send + 'static,
    F: AsyncTxFn + Sync + Send + 'static,
    F::In: DeserializeOwned,
    F::Out: Serialize,
    F::E: Serialize,
    MF: Make<F> + 'static,
{
    use super::_axum_handler_type_checker_2_args_generic;

    _axum_handler_type_checker_2_args_generic::<_, Json<F::In>, _, _, ()>(
        &handler_tx_headers_old::<F, MF, TL>,
    );
}
