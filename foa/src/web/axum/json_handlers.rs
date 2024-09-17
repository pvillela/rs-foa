use crate::{
    context::LocaleSelf,
    fun::{AsyncFn, AsyncFn2},
};
use axum::{
    extract::{FromRequestParts, Request},
    handler::Handler,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

//=================
// Type checker

// #[cfg(test)]
use axum::extract::FromRequest;

#[cfg(test)]
/// Checks a closure for compliance with Axum Handler impl requirements.
pub fn _axum_handler_type_checker_2_args_generic<In1, In2, Out, Fut, S>(
    _f: &(impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static),
) where
    Fut: Future<Output = Out> + Send,
    In1: FromRequestParts<S> + Send,
    In2: FromRequest<S> + Send,
    Out: IntoResponse,
    S: Send + Sync + 'static,
{
}

//=================
// Simple handler of async fn

pub fn handler_fn<In, Out, Fut>(
    f: impl FnOnce(In) -> Fut + Clone + Send + 'static,
) -> impl FnOnce(Json<In>) -> Fut + Clone + Send + 'static
where
    In: DeserializeOwned,
    Out: IntoResponse,
    Fut: Future<Output = Out> + Send,
{
    move |Json(input)| f(input)
}

pub fn handler_fn2<In1, In2, Out, E, Fut, S>(
    f: impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static,
) -> impl FnOnce(In1, Json<In2>) -> Fut + Clone + Send + 'static
where
    In1: FromRequestParts<S>,
    In2: DeserializeOwned,
    Out: IntoResponse,
    E: IntoResponse,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<Out, E>> + Send,
{
    move |in1, Json(in2)| f(in1, in2)
}

#[cfg(test)]
fn _typecheck_handler_fn2<In1, In2, Out, E, Fut, S>(
    f: impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static,
) where
    In1: FromRequestParts<S> + Send,
    In2: DeserializeOwned + Send,
    Out: IntoResponse,
    E: IntoResponse,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<Out, E>> + Send,
{
    _axum_handler_type_checker_2_args_generic::<_, Json<In2>, _, _, S>(&handler_fn2(f));
}

//=================
// LocaleSelf for Parts

impl LocaleSelf for Parts {
    fn locale(&self) -> Option<&str> {
        self.headers.get("Accept-Language")?.to_str().ok()
    }
}

//=================
// Handler for Fn(In1, In2) -> Future<Output = Result<Json<Out>, Json<E>>

pub fn handler_fn2r<In1, In2, Out, E, Fut, S>(
    f: impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static,
) -> impl Fn(
    In1,
    Json<In2>,
) -> Pin<Box<(dyn Future<Output = (StatusCode, Json<Result<Out, E>>)> + Send + 'static)>>
       + Send
       + 'static
       + Clone
where
    In1: FromRequestParts<S> + Send + 'static,
    In2: DeserializeOwned + Send + 'static,
    Result<Out, E>: Serialize,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<Out, E>> + Send,
{
    move |req_part, Json(input)| {
        let f = f.clone();
        Box::pin(async move {
            let out = f(req_part, input).await;
            let status = match out {
                Ok(_) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(out))
        })
    }
}

#[cfg(test)]
fn _typecheck_handler_fn2r<In1, In2, Out, E, Fut, S>(
    f: impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static,
) where
    In1: FromRequestParts<S> + Send + 'static,
    In2: DeserializeOwned + Send + 'static,
    Out: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<Out, E>> + Send,
{
    _axum_handler_type_checker_2_args_generic::<_, Json<In2>, _, _, S>(&handler_fn2r(f));
}

//=================
// Handlers for AsyncFn[x]

pub fn handler_asyncfn2<F, S>(
    f: F,
) -> impl Fn(F::In1, Json<F::In2>) -> Pin<Box<(dyn Future<Output = Json<F::Out>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncFn2 + Send + Sync + Clone + 'static,
    F::In1: FromRequestParts<S> + Send,
    F::In2: DeserializeOwned + Send,
    F::Out: Serialize,
    S: Send + Sync + 'static,
{
    move |req_part, Json(input)| {
        let f = f.clone();
        Box::pin(async move { Json(f.invoke(req_part, input).await) })
    }
}

pub fn handler_asyncfn2_arc<F, S>(
    f: F,
) -> impl Fn(F::In1, Json<F::In2>) -> Pin<Box<(dyn Future<Output = Json<F::Out>> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncFn2 + Send + Sync + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    F::Out: Serialize,
    S: Send + Sync + 'static,
{
    handler_asyncfn2(Arc::new(f))
}

pub fn handler_asyncfn2r<O, E, F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<Box<(dyn Future<Output = (StatusCode, Json<F::Out>)> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + Clone + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    F::Out: Serialize,
    S: Send + Sync + 'static,
{
    handler_fn2r(f.into_fnonce_when_clone())
}

pub fn handler_asyncfn2r_arc<O, E, F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<Box<(dyn Future<Output = (StatusCode, Json<F::Out>)> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static,
    F::In1: FromRequestParts<S> + Send,
    F::In2: DeserializeOwned + Send,
    O: Serialize + Send,
    E: Serialize + Send,
    S: Send + Sync + 'static,
{
    handler_asyncfn2r(Arc::new(f))
}

#[derive(Clone)]
pub struct HandlerAsyncFn2r<F>(pub F);

pub type HandlerAsyncFn2rArc<F> = HandlerAsyncFn2r<Arc<F>>;

impl<F> HandlerAsyncFn2rArc<F> {
    pub fn new(f: F) -> Self {
        HandlerAsyncFn2r(f.into())
    }
}

impl<O, E, F, S> Handler<(), S> for HandlerAsyncFn2r<F>
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static + Clone,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    O: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, req: Request, state: S) -> Self::Future {
        handler_asyncfn2r::<O, E, F, S>(self.0).call(req, state)
    }
}

pub fn handler_asyncfn2_json<In2, O, E, F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    F::In2,
) -> Pin<
    Box<(dyn Future<Output = Result<Json<O>, (StatusCode, Json<E>)>> + Send + 'static)>,
>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncFn2<In2 = Json<In2>, Out = Result<Json<O>, (StatusCode, Json<E>)>>
        + Send
        + Sync
        + Clone
        + 'static,
    F::In1: FromRequestParts<S>,
    In2: DeserializeOwned + Send + 'static,
    E: Serialize,
    S: Send + Sync + 'static,
{
    move |req_part, json| {
        let f = f.clone();
        Box::pin(async move { f.invoke(req_part, json).await })
    }
}

pub fn handler_asyncfn2_general<F, S, M>(
    f: F,
) -> impl Fn(F::In1, F::In2) -> Pin<Box<(dyn Future<Output = F::Out> + Send + 'static)>>
       + Send
       + Sync // not needed for Axum
       + 'static
       + Clone
where
    F: AsyncFn2 + Send + Sync + Clone + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: FromRequest<S, M> + Send + 'static,
    S: Send + Sync + 'static,
{
    move |req_part, json| {
        let f = f.clone();
        Box::pin(async move { f.invoke(req_part, json).await })
    }
}

pub struct HandlerAsyncFn2rWithErrorMapper<F, M, EMI, EMO>(pub F, pub M, PhantomData<(EMI, EMO)>);

impl<F, M, EMI, EMO> Clone for HandlerAsyncFn2rWithErrorMapper<F, M, EMI, EMO>
where
    F: Clone,
    M: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<O, E, EMI, EMO, F, Mpr, S> Handler<(), S> for HandlerAsyncFn2rWithErrorMapper<F, Mpr, EMI, EMO>
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static + Clone,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    O: Serialize + Send,
    E: Serialize + Into<EMI> + Send,
    S: Send + Sync + 'static,
    Mpr: Fn(EMI) -> (StatusCode, EMO) + Send + Sync + 'static + Clone,
    EMI: Send + 'static + Sync,
    EMO: Serialize + Send + 'static + Sync,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, req: Request, state: S) -> Self::Future {
        let mf = WithMappedErrors(self.0, self.1, PhantomData);
        // let h = handler_with_error_mapping(mf);
        let h = handler_asyncfn2_general(mf);
        // Handler::<_, S>::call(h, req, state)
        h.call(req, state)
    }
}

struct WithMappedErrors<EMI, EMO, F, M>(F, M, PhantomData<(EMI, EMO)>);

impl<EMI, EMO, F: Clone, M: Clone> Clone for WithMappedErrors<EMI, EMO, F, M> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<O, E, EMI, EMO, F, Mpr> AsyncFn2 for WithMappedErrors<EMI, EMO, F, Mpr>
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static,
    F::In2: DeserializeOwned,
    O: Serialize + Send,
    E: Serialize + Into<EMI> + Send,
    Mpr: Fn(EMI) -> (StatusCode, EMO) + Send + Sync + 'static,
    EMI: Send + Sync,
    EMO: Send + Sync,
{
    type In1 = F::In1;
    type In2 = Json<F::In2>;
    type Out = Result<Json<O>, (StatusCode, Json<EMO>)>;

    async fn invoke(&self, in1: Self::In1, Json(in2): Self::In2) -> Self::Out {
        let out_f = self.0.invoke(in1, in2).await;
        match out_f {
            Ok(out) => Ok(Json(out)),
            Err(err) => {
                let (status_code, err) = self.1(err.into());
                Err((status_code, Json(err)))
            }
        }
    }
}
