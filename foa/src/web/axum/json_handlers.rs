use crate::{
    context::LocaleSelf,
    db::sqlx::{in_tx, AsyncTxFn, DbCtx},
    fun::Async2RFn,
    tokio::task_local::{invoke_tl_scoped, TaskLocal, TaskLocalCtx},
    trait_utils::Make,
};
use axum::{extract::FromRequestParts, http::HeaderMap, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::{future::Future, marker::PhantomData, ops::Deref, pin::Pin, sync::Arc};

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

pub fn handler_async2rfn<W, S>(
    w: W,
) -> impl Fn(
    W::In1,
    Json<W::In2>,
) -> Pin<Box<(dyn Future<Output = Result<Json<W::Out>, Json<W::E>>> + Send + 'static)>>
       + Send
       + Sync
       + 'static
       + Clone
where
    W: Async2RFn + Send + Sync + Clone + 'static,
    W::In1: FromRequestParts<S>,
    W::In2: Deserialize<'static> + 'static,
    W::Out: Serialize,
    W::E: Serialize,
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

//=================
// Handler for AsyncTxFn

// ... TBD ...

//=================
// Handler for AsyncRFn in task-local context

impl LocaleSelf for HeaderMap {
    fn locale(&self) -> &str {
        let header_value = self.get("Accept-Language");
        match header_value {
            None => "en-CA",
            Some(v) => v.to_str().unwrap_or("en-CA"),
        }
    }
}

// ... TBD ...

//=================
// Handler for AsyncTxFn in task-local context

struct HandlerTxHeadersFn<CTX, F, D>(Arc<F>, PhantomData<(CTX, D)>);

impl<CTX, F, D> Clone for HandlerTxHeadersFn<CTX, F, D> {
    fn clone(&self) -> Self {
        HandlerTxHeadersFn(self.0.clone(), PhantomData)
    }
}

impl<CTX, F, D> HandlerTxHeadersFn<CTX, F, D> {
    fn new(f: F) -> Self {
        Self(f.into(), PhantomData)
    }
}

impl<CTX, F, D> Async2RFn for HandlerTxHeadersFn<CTX, F, D>
where
    CTX: DbCtx + TaskLocalCtx<D> + Sync + Send + 'static,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncTxFn<CTX> + Sync + Send + 'static,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync + Send + 'static,
{
    type In1 = HeaderMap;
    type In2 = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, headers: HeaderMap, input: F::In) -> Result<Self::Out, Self::E> {
        let f_in_tx = in_tx(self.0.deref()).await;
        let output = invoke_tl_scoped::<CTX, _, D>(&f_in_tx, (headers, input)).await?;
        Ok(output)
    }
}

pub fn handler_tx_headers<CTX, F, D, S>(
    f: F,
) -> impl Fn(
    HeaderMap,
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    CTX: DbCtx + TaskLocalCtx<D> + Sync + Send + 'static,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncTxFn<CTX> + Sync + Send + 'static,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync + Send + 'static,
    S: Send + Sync + 'static,
{
    let wf = HandlerTxHeadersFn::new(f);
    handler_async2rfn::<_, S>(wf)
}

#[deprecated]
pub async fn handler_tx_headers_old<CTX, F, MF, D>(
    headers: HeaderMap,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: DbCtx + TaskLocalCtx<D> + Sync + 'static,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncTxFn<CTX> + Sync,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync,
    MF: Make<F>,
{
    let f = MF::make();
    let f_in_tx = in_tx(&f).await;
    let output = invoke_tl_scoped::<CTX, _, D>(&f_in_tx, (headers, input)).await?;
    Ok(Json(output))
}
