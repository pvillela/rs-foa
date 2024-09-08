use crate::{
    context::LocaleSelf,
    db::sqlx::{in_tx_borrowed, AsyncTxFn, DbCtx},
    fun::{Async2RFn, Async3RFn, Async4RFn, AsyncRFn},
    tokio::task_local::{
        invoke_tl_scoped, Async2RFnTlD, Async3RFnTlD, Async4RFnTlD, TaskLocal, TaskLocalCtx,
    },
    trait_utils::Make,
    wrapper::W,
};
use axum::{extract::FromRequestParts, http::HeaderMap, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin, sync::Arc};

//=================
// Type checker

/// Checks a closure for compliance with Axum Handler impl requirements.
fn _axum_handler_type_checker_2_args_generic<In1, In2, Out, E, Fut, S>(
    _f: &(impl FnOnce(In1, Json<In2>) -> Fut + Clone + Send + 'static),
) where
    Fut: Future<Output = Result<Json<Out>, Json<E>>> + Send,
    In1: FromRequestParts<S>,
    In2: Deserialize<'static> + 'static,
    Out: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
{
}

//=================
// To be updated

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

//=================
// Handlers for Async[x]RFn

pub fn handler_asyncrfn<F>(
    w: F,
) -> impl Fn(
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncRFn + Send + Sync + Clone + 'static,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    move |Json(input)| {
        let f = w.clone();
        Box::pin(async move {
            let output = f.invoke(input).await?;
            Ok(Json(output))
        })
    }
}

pub fn handler_async2rfn<F, S>(
    w: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: Async2RFn + Send + Sync + Clone + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    S: Send + Sync + 'static,
{
    move |req_part, Json(input)| {
        let f = w.clone();
        Box::pin(async move {
            let output = f.invoke(req_part, input).await?;
            Ok(Json(output))
        })
    }
}

pub fn handler_async3rfn<F, S>(
    w: F,
) -> impl Fn(
    F::In1,
    F::In2,
    Json<F::In3>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: Async3RFn + Send + Sync + Clone + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: FromRequestParts<S>,
    F::In3: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    S: Send + Sync + 'static,
{
    move |req_part1, req_part2, Json(input)| {
        let f = w.clone();
        Box::pin(async move {
            let output = f.invoke(req_part1, req_part2, input).await?;
            Ok(Json(output))
        })
    }
}

pub fn handler_async4rfn<F, S>(
    w: F,
) -> impl Fn(
    F::In1,
    F::In2,
    F::In3,
    Json<F::In4>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: Async4RFn + Send + Sync + Clone + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: FromRequestParts<S>,
    F::In3: FromRequestParts<S>,
    F::In4: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    S: Send + Sync + 'static,
{
    move |req_part1, req_part2, req_part3, Json(input)| {
        let f = w.clone();
        Box::pin(async move {
            let output = f.invoke(req_part1, req_part2, req_part3, input).await?;
            Ok(Json(output))
        })
    }
}

//=================
// LocaleSelf for HeaderMap

impl LocaleSelf for HeaderMap {
    fn locale(&self) -> &str {
        let header_value = self.get("Accept-Language");
        match header_value {
            None => "en-CA",
            Some(v) => v.to_str().unwrap_or("en-CA"),
        }
    }
}

//=================
// Handler for AsyncTxFn

pub fn handler_tx<CTX, F>(
    f: F,
) -> impl Fn(
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    CTX: DbCtx + Sync + Send + 'static,
    F: AsyncTxFn<CTX> + Sync + Send + 'static,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    let wf = Arc::new(f.in_tx());
    handler_asyncrfn(wf)
}

//=================
// Handlers for AsyncTxFn in task-local context

pub fn handler_tx_1requestpart<CTX, F, RP, S>(
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
    CTX: DbCtx + TaskLocalCtx + Sync + Send + 'static,
    CTX::TaskLocal: TaskLocal<ValueType = RP>,
    F: AsyncTxFn<CTX> + Sync + Send + 'static,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    RP: FromRequestParts<S> + Sync + Send + 'static,
    S: Send + Sync + 'static,
{
    let wf1 = f.in_tx();
    let wf2 = W::<_, Async2RFnTlD, CTX>::new(Arc::new(wf1));
    handler_async2rfn(wf2)
}

pub fn handler_tx_2requestparts<CTX, F, RP1, RP2, S>(
    f: F,
) -> impl Fn(
    RP1,
    RP2,
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    CTX: DbCtx + TaskLocalCtx + Sync + Send + 'static,
    CTX::TaskLocal: TaskLocal<ValueType = (RP1, RP2)>,
    F: AsyncTxFn<CTX> + Sync + Send + 'static,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    RP1: FromRequestParts<S> + Sync + Send + 'static,
    RP2: FromRequestParts<S> + Sync + Send + 'static,
    S: Send + Sync + 'static,
{
    let wf1 = f.in_tx();
    let wf2 = W::<_, Async3RFnTlD, CTX>::new(Arc::new(wf1));
    handler_async3rfn(wf2)
}

pub fn handler_tx_3requestparts<CTX, F, RP1, RP2, RP3, S>(
    f: F,
) -> impl Fn(
    RP1,
    RP2,
    RP3,
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    CTX: DbCtx + TaskLocalCtx + Sync + Send + 'static,
    CTX::TaskLocal: TaskLocal<ValueType = (RP1, RP2, RP3)>,
    F: AsyncTxFn<CTX> + Sync + Send + 'static,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    RP1: FromRequestParts<S> + Sync + Send + 'static,
    RP2: FromRequestParts<S> + Sync + Send + 'static,
    RP3: FromRequestParts<S> + Sync + Send + 'static,
    S: Send + Sync + 'static,
{
    let wf1 = f.in_tx();
    let wf2 = W::<_, Async4RFnTlD, CTX>::new(Arc::new(wf1));
    handler_async4rfn(wf2)
}

#[deprecated]
pub async fn handler_tx_headers_old<CTX, F, MF>(
    headers: HeaderMap,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: DbCtx + TaskLocalCtx + Sync + 'static,
    CTX::TaskLocal: TaskLocal<ValueType = HeaderMap>,
    F: AsyncTxFn<CTX> + Sync,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    MF: Make<F>,
{
    let f = MF::make();
    let f_in_tx = in_tx_borrowed(&f).await;
    let output = invoke_tl_scoped::<CTX, _>(&f_in_tx, (headers, input)).await?;
    Ok(Json(output))
}
