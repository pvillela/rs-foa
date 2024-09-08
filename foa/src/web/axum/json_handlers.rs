use crate::{
    context::LocaleSelf,
    db::sqlx::{in_tx, AsyncTxFn, DbCtx},
    fun::{Async2RFn, AsyncRFn},
    tokio::task_local::{invoke_tl_scoped, TaskLocal, TaskLocalCtx},
    trait_utils::Make,
};
use axum::{extract::FromRequestParts, http::HeaderMap, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

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

pub fn handler_asyncrfn<W>(
    w: W,
) -> impl Fn(
    Json<W::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<W::Out>, Json<W::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    W: AsyncRFn + Send + Sync + Clone + 'static,
    W::In: Deserialize<'static> + 'static,
    W::Out: Serialize,
    W::E: Serialize,
{
    move |Json(input)| {
        let f = w.clone();
        Box::pin(async move {
            let output = f.invoke(input).await?;
            Ok(Json(output))
        })
    }
}

pub fn handler_async2rfn<W, S>(
    w: W,
) -> impl Fn(
    W::In1,
    Json<W::In2>,
) -> Pin<Box<(dyn Future<Output = Result<Json<W::Out>, Json<W::E>>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
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
// Handler for AsyncTxFn

impl<CTX, F> AsyncRFn for (Arc<F>, PhantomData<CTX>)
where
    CTX: DbCtx + Sync + Send,
    F: AsyncTxFn<CTX> + Sync + Send,
{
    type In = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, input: F::In) -> Result<Self::Out, Self::E> {
        let output = self.0.as_ref().invoke_in_tx(input).await?;
        Ok(output)
    }
}

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
    let wf = (Arc::new(f), PhantomData::<CTX>);
    handler_asyncrfn(wf)
}

//=================
// Handler for AsyncTxFn in task-local context

impl<CTX, F, RP, S> Async2RFn for (Arc<F>, PhantomData<(CTX, RP, S)>)
where
    CTX: DbCtx + TaskLocalCtx + Sync + Send + 'static,
    CTX::TaskLocal: TaskLocal<ValueType = RP>,
    F: AsyncTxFn<CTX> + Sync + Send,
    RP: FromRequestParts<S> + Sync + Send,
    S: Send + Sync,
{
    type In1 = RP;
    type In2 = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, rp: RP, input: F::In) -> Result<Self::Out, Self::E> {
        let f_in_tx = self.0.as_ref().in_tx();
        let output = invoke_tl_scoped::<CTX, _>(&f_in_tx, (rp, input)).await?;
        Ok(output)
    }
}

pub fn handler_tx_requestpart<CTX, F, RP, S>(
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
    let wf = (Arc::new(f), PhantomData::<(CTX, RP, S)>);
    handler_async2rfn::<(Arc<F>, PhantomData<(CTX, RP, S)>), S>(wf)
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
    let f_in_tx = in_tx(&f).await;
    let output = invoke_tl_scoped::<CTX, _>(&f_in_tx, (headers, input)).await?;
    Ok(Json(output))
}
