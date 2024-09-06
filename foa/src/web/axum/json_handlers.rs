use crate::{
    context::LocaleSelf,
    db::sqlx::{in_tx, invoke_in_tx, AsyncTxFn, DbCtx},
    fun::{Async2RFn, AsyncRFn},
    tokio::task_local::{invoke_tl_scoped, TaskLocal, TaskLocalCtx},
    trait_utils::Make,
};
use axum::{http::HeaderMap, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::{future::Future, marker::PhantomData, ops::Deref, pin::Pin, sync::Arc};

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
    let output = invoke_in_tx(&MF::make(), input).await?;
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
    let output = invoke_tl_scoped::<CTX, F, D>(&MF::make(), (headers, input)).await?;
    Ok(Json(output))
}

//=================
// Handler for F: AsyncTxFn in task-local context

pub struct HandlerTxHeadersFn<CTX, F, D>(Arc<F>, PhantomData<(CTX, D)>);

impl<CTX, F, D> Clone for HandlerTxHeadersFn<CTX, F, D> {
    fn clone(&self) -> Self {
        HandlerTxHeadersFn(self.0.clone(), PhantomData)
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
    type In2 = Json<F::In>;
    type Out = Json<F::Out>;
    type E = Json<F::E>;

    async fn invoke(
        &self,
        headers: HeaderMap,
        Json(input): Json<F::In>,
    ) -> Result<Self::Out, Self::E> {
        let f_in_tx = in_tx(self.0.deref()).await;
        let output = invoke_tl_scoped::<CTX, _, D>(&f_in_tx, (headers, input)).await?;
        Ok(Json(output))
    }
}

pub fn handler_tx_headers_fn<CTX, F, D>(f: F) -> HandlerTxHeadersFn<CTX, F, D>
where
    CTX: DbCtx + TaskLocalCtx<D> + Sync + Send + 'static,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncTxFn<CTX> + Sync + Send + 'static,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync + Send + 'static,
{
    HandlerTxHeadersFn(f.into(), PhantomData::<(CTX, D)>)
}

fn _axum_handler_type_checker_2_args_generic<S1, S2, T, E, Fut>(
    _f: &(impl FnOnce(S1, Json<S2>) -> Fut + Clone + Send + 'static),
) where
    Fut: Future<Output = Result<Json<T>, Json<E>>> + Send,
    S2: Deserialize<'static> + 'static,
    T: Serialize,
    E: Serialize,
{
}

async fn _axum_handler_type_checker<CTX, F, D>(f: F)
where
    CTX: DbCtx + TaskLocalCtx<D> + Sync + Send + 'static,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncTxFn<CTX> + Sync + Send + 'static,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync + Send + 'static,
{
    let g1 = handler_tx_headers_fn(f);
    let g2 = |headers, json| async move { g1.invoke(headers, json).await };
    _axum_handler_type_checker_2_args_generic(&g2);
}

// impl<CTX, F, D, M, S> Handler<(M, HeaderMap, Json<F::In>), S> for HandlerTxHeadersFn<CTX, F, D>
// where
//     CTX: DbCtx + TaskLocalCtx<D> + Sync + Send + 'static,
//     CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
//     F: AsyncTxFn<CTX> + Sync + Send + 'static,
//     F::In: Deserialize<'static> + 'static,
//     F::Out: Serialize,
//     F::E: Serialize,
//     D: Sync + Send + 'static,
//     S: Send + Sync + 'static,
// {
//     type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

//     fn call(self, req: axum::extract::Request, state: S) -> Self::Future {
//         let g1 = self;
//         let g2 = |headers, json| Box::pin(async move { g1.invoke(headers, json).await });
//         // let g3 = Box::new(g2);
//         // Handler::call(g3, req, state)
//         Handler::call(g2, req, state)
//     }
// }

fn handler_of_wf<CTX, F, D>(
    wf: HandlerTxHeadersFn<CTX, F, D>,
) -> impl Fn(
    HeaderMap,
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync
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
{
    move |headers, json| {
        let f = wf.clone();
        Box::pin(async move { f.invoke(headers, json).await })
    }
}

pub fn handler_of_f_headers<CTX, F, D>(
    f: F,
) -> impl Fn(
    HeaderMap,
    Json<F::In>,
) -> Pin<Box<(dyn Future<Output = Result<Json<F::Out>, Json<F::E>>> + Send + 'static)>>
       + Send
       + Sync
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
{
    let wf = HandlerTxHeadersFn(f.into(), PhantomData::<(CTX, D)>);
    handler_of_wf(wf)
}

pub async fn handler_tx_headers<CTX, F, MF, D>(
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
