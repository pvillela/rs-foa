use crate::{
    context::{Itself, LocaleSelf},
    db::sqlx::{in_tx, invoke_in_tx, AsyncTxFn, DbCtx},
    fun::{Async2RFn, AsyncRFn},
    tokio::task_local::{invoke_tl_scoped, TaskLocal, TaskLocalCtx},
};
use axum::{http::HeaderMap, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::{future::Future, marker::PhantomData};

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
// Handler for F: AsyncRFn

struct HandlerRFn<F>(F);

impl<F> AsyncRFn for HandlerRFn<F>
where
    F: AsyncRFn + Sync,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    type In = Json<F::In>;
    type Out = Json<F::Out>;
    type E = Json<F::E>;

    async fn invoke(&self, Json(input): Json<F::In>) -> Result<Self::Out, Self::E> {
        let output = self.0.invoke(input).await?;
        Ok(Json(output))
    }
}

pub async fn handler_r_fn<F>(
    f: F,
) -> impl AsyncRFn<In = Json<F::In>, Out = Json<F::Out>, E = Json<F::E>>
where
    F: AsyncRFn + Sync,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    HandlerRFn(f)
}

pub async fn handler<F>(Json(input): Json<F::In>) -> Result<Json<F::Out>, Json<F::E>>
where
    F: AsyncRFn + Itself,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    let output = F::it().invoke(input).await?;
    Ok(Json(output))
}

//=================
// Handler for F: AsyncTxFn

struct HandlerTxFn<CTX, F>(F, PhantomData<CTX>);

impl<CTX, F> AsyncRFn for HandlerTxFn<CTX, F>
where
    CTX: DbCtx + Sync + Send,
    F: AsyncTxFn<CTX> + Sync + Send + Clone,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    type In = Json<F::In>;
    type Out = Json<F::Out>;
    type E = Json<F::E>;

    async fn invoke(&self, Json(input): Json<F::In>) -> Result<Self::Out, Self::E> {
        let output = invoke_in_tx(self.0.clone(), input).await?;
        Ok(Json(output))
    }
}

pub async fn handler_tx_fn<CTX, F>(
    f: F,
) -> impl AsyncRFn<In = Json<F::In>, Out = Json<F::Out>, E = Json<F::E>>
where
    CTX: DbCtx + Sync + Send,
    F: AsyncTxFn<CTX> + Sync + Send + Clone,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    HandlerTxFn(f, PhantomData)
}

pub async fn handler_tx<CTX, F>(Json(input): Json<F::In>) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: DbCtx + Sync,
    F: AsyncTxFn<CTX> + Itself + Sync,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
{
    let output = invoke_in_tx(F::it(), input).await?;
    Ok(Json(output))
}

//=================
// Handler for F: AsyncRFn in task-local context

impl LocaleSelf for HeaderMap {
    fn locale(&self) -> &str {
        let header_value = self.get("Accept-Language");
        match header_value {
            None => "en-CA",
            Some(v) => v.to_str().unwrap_or("en-CA"),
        }
    }
}

struct HandlerHeadersFn<CTX, F, D>(F, PhantomData<(CTX, D)>);

impl<CTX, F, D> Async2RFn for HandlerHeadersFn<CTX, F, D>
where
    CTX: TaskLocalCtx<D> + Sync + Send,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncRFn + Sync + Send + Clone,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync + Send,
{
    type In1 = HeaderMap;
    type In2 = Json<F::In>;
    type Out = Json<F::Out>;
    type E = Json<F::E>;

    async fn invoke(
        &self,
        headers: HeaderMap,
        Json(input): Json<F::In>,
    ) -> Result<Json<F::Out>, Json<F::E>> {
        let output = invoke_tl_scoped::<CTX, F, D>(self.0.clone(), (headers, input)).await?;
        Ok(Json(output))
    }
}

pub async fn handler_headers_fn<CTX, F, D>(
    f: F,
) -> impl Async2RFn<In1 = HeaderMap, In2 = Json<F::In>, Out = Json<F::Out>, E = Json<F::E>>
where
    CTX: TaskLocalCtx<D> + Sync + Send,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncRFn + Sync + Send + Clone,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync + Send,
{
    HandlerHeadersFn(f, PhantomData::<(CTX, D)>)
}

pub async fn handler_headers<CTX, F, D>(
    headers: HeaderMap,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: TaskLocalCtx<D> + Sync,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncRFn + Itself + Sync,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync,
{
    let output = invoke_tl_scoped::<CTX, F, D>(F::it(), (headers, input)).await?;
    Ok(Json(output))
}

//=================
// Handler for F: AsyncTxFn in task-local context

#[derive(Clone)]
struct PreHdlrTxHeadersFn<CTX, F, D>(F, PhantomData<(CTX, D)>);

impl<CTX, F, D> Async2RFn for PreHdlrTxHeadersFn<CTX, F, D>
where
    CTX: DbCtx + TaskLocalCtx<D> + Sync + Send + Clone,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap> + Clone,
    F: AsyncTxFn<CTX> + Sync + Send + Clone,
    F::In: Deserialize<'static> + 'static + Clone,
    F::Out: Serialize + Clone,
    F::E: Serialize + Clone,
    D: Sync + Send + Clone,
{
    type In1 = HeaderMap;
    type In2 = F::In;
    type Out = Json<F::Out>;
    type E = Json<F::E>;

    async fn invoke(&self, headers: HeaderMap, input: F::In) -> Result<Self::Out, Self::E> {
        let f_in_tx = in_tx(self.0.clone()).await;
        let output = invoke_tl_scoped::<CTX, _, _>(f_in_tx, (headers, input)).await?;
        Ok(Json(output))
    }
}

pub fn pre_hdlr_tx_headers<CTX, F, D>(
    f: F,
) -> impl Async2RFn<In1 = HeaderMap, In2 = F::In, Out = Json<F::Out>, E = Json<F::E>> + Clone
where
    CTX: DbCtx + TaskLocalCtx<D> + Sync + Send + Clone,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap> + Clone,
    F: AsyncTxFn<CTX> + Sync + Send + Clone,
    F::In: Deserialize<'static> + 'static + Clone,
    F::Out: Serialize + Clone,
    F::E: Serialize + Clone,
    D: Sync + Send + Clone,
{
    PreHdlrTxHeadersFn(f, PhantomData::<(CTX, D)>)
}

pub fn handler_of_2r_oj<S1, S2, T, E, Fut>(
    f: impl FnOnce(S1, S2) -> Fut + Clone + Send + 'static,
) -> impl FnOnce(S1, Json<S2>) -> Fut + Clone + Send + 'static
where
    Fut: Future<Output = Result<Json<T>, Json<E>>> + Send,
    S2: Deserialize<'static> + 'static,
    T: Serialize,
    E: Serialize,
{
    move |input1, Json(input2)| f(input1, input2)
}

/// Check types for compatibility with Axum handlers
fn _axum_handler_type_checker<CTX, F, D>(f: F)
where
    CTX: DbCtx + TaskLocalCtx<D> + Sync + Send + Clone + 'static,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap> + Clone,
    F: AsyncTxFn<CTX> + Sync + Send + Clone + 'static,
    F::In: Deserialize<'static> + 'static + Clone,
    F::Out: Serialize + Clone,
    F::E: Serialize + Clone,
    D: Sync + Send + Clone + 'static,
{
    let pre = pre_hdlr_tx_headers(f);
    let f1 = move |headers, input| async move { pre.invoke(headers, input).await };
    let _f2 = handler_of_2r_oj(f1);
}

pub async fn handler_tx_headers<CTX, F, D>(
    headers: HeaderMap,
    Json(input): Json<F::In>,
) -> Result<Json<F::Out>, Json<F::E>>
where
    CTX: DbCtx + TaskLocalCtx<D> + Sync,
    CTX::TaskLocal: TaskLocal<D, ValueType = HeaderMap>,
    F: AsyncTxFn<CTX> + Itself + Sync,
    F::In: Deserialize<'static> + 'static,
    F::Out: Serialize,
    F::E: Serialize,
    D: Sync,
{
    let f_in_tx = in_tx(F::it()).await;
    let output = invoke_tl_scoped::<CTX, _, _>(f_in_tx, (headers, input)).await?;
    Ok(Json(output))
}
