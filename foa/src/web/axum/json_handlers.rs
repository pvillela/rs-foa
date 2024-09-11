use crate::{
    context::LocaleSelf,
    db::sqlx::{in_tx, AsyncTxFn},
    fun::{AsyncRFn, AsyncRFn2},
    tokio::task_local::{invoke_tl_scoped, TaskLocal},
    trait_utils::Make,
};
use axum::{extract::FromRequestParts, http::request::Parts, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin, sync::Arc};

//=================
// Type checker

#[cfg(test)]
/// Checks a closure for compliance with Axum Handler impl requirements.
fn _axum_handler_type_checker_2_args_generic<In1, In2, Out, E, Fut, S>(
    _f: &(impl FnOnce(In1, Json<In2>) -> Fut + Clone + Send + 'static),
) where
    Fut: Future<Output = Result<Json<Out>, Json<E>>> + Send,
    In1: FromRequestParts<S> + Send,
    In2: Deserialize<'static> + Send,
    Out: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
{
}

//=================
// Simple handler of async fn

pub fn handler_of<In, Out, Fut>(
    f: impl FnOnce(In) -> Fut + Clone + Send + 'static,
) -> impl FnOnce(Json<In>) -> Fut + Clone + Send + 'static
where
    In: Deserialize<'static>,
    Out: IntoResponse,
    Fut: Future<Output = Out> + Send,
{
    move |Json(input)| f(input)
}

//=================
// Handlers for Async[x]RFn

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
    F::In: Deserialize<'static>,
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
    F::In: Deserialize<'static>,
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
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncRFn2 + Send + Sync + Clone + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: Deserialize<'static>,
    F::Out: Serialize,
    F::E: Serialize,
    S: Send + Sync + 'static,
{
    move |req_part, Json(input)| {
        let f = f.clone();
        Box::pin(async move {
            let output = f.invoke(req_part, input).await?;
            Ok(Json(output))
        })
    }
}

pub fn handler_asyncrfn2_arc<F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncRFn2 + Send + Sync + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: Deserialize<'static>,
    F::Out: Serialize,
    F::E: Serialize,
    S: Send + Sync + 'static,
{
    handler_asyncrfn2(Arc::new(f))
}

//=================
// LocaleSelf for Parts

impl LocaleSelf for Parts {
    fn locale(&self) -> Option<&str> {
        self.headers.get("Accept-Language")?.to_str().ok()
    }
}

//=================
// Handler for AsyncTxFn

pub fn handler_tx<F>(
    f: F,
) -> impl Fn(
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncTxFn + Sync + Send + 'static,
    F::In: Deserialize<'static>,
    F::Out: Serialize,
    F::E: Serialize,
{
    handler_asyncrfn_arc(f.in_tx())
}

//=================
// Handler for AsyncTxFn in task-local context

pub fn handler_tx_requestparts<F, RP, S, TL>(
    f: F,
) -> impl Fn(
    RP,
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    TL: TaskLocal<ValueType = RP> + Sync + Send + 'static,
    F: AsyncTxFn + Sync + Send + 'static,
    F::In: Deserialize<'static>,
    F::Out: Serialize,
    F::E: Serialize,
    RP: FromRequestParts<S> + Send,
    S: Send + Sync + 'static,
{
    handler_asyncrfn2_arc(f.in_tx_tl_scoped::<TL>())
}

#[deprecated]
pub async fn handler_tx_headers_old<F, MF, TL>(
    parts: Parts,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    TL: TaskLocal<ValueType = Parts> + Sync + Send + 'static,
    F: AsyncTxFn + Sync + Send + 'static,
    F::In: Deserialize<'static>,
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
    TL: TaskLocal<ValueType = Parts> + Sync + Send + 'static,
    F: AsyncTxFn + Sync + Send + 'static,
    F::In: Deserialize<'static>,
    F::Out: Serialize,
    F::E: Serialize,
    MF: Make<F> + 'static,
{
    _axum_handler_type_checker_2_args_generic::<_, _, _, _, _, ()>(
        &handler_tx_headers_old::<F, MF, TL>,
    );
}
