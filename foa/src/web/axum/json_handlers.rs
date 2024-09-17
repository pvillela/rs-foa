use crate::fun::AsyncFn2;
use axum::{
    extract::{FromRequestParts, Request},
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use super::WithMappedErrors;

/// Transforms an [`FnOnce`] to take [`Json`] input, making it into a [`Handler`].
pub fn into_json_input_fn<In, Out, Fut>(
    f: impl FnOnce(In) -> Fut + Clone + Send + 'static,
) -> impl FnOnce(Json<In>) -> Fut + Clone + Send + 'static
where
    In: DeserializeOwned,
    Out: IntoResponse,
    Fut: Future<Output = Out> + Send,
{
    move |Json(input)| f(input)
}

/// Transforms an [`FnOnce`] to take [`Json`] input, making it into a [`Handler`].
pub fn into_json_input_fn2<In1, In2, Out, E, Fut, S>(
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
fn _typecheck_into_json_input_fn2<In1, In2, Out, E, Fut, S>(
    f: impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static,
) where
    In1: FromRequestParts<S> + Send,
    In2: DeserializeOwned + Send,
    Out: IntoResponse,
    E: IntoResponse,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<Out, E>> + Send,
{
    use super::_axum_handler_type_checker_2_args_generic;

    _axum_handler_type_checker_2_args_generic::<_, Json<In2>, _, _, S>(&into_json_input_fn2(f));
}

/// Returns a handler for `Fn(In1, In2) -> Future<Output = Result<O, E>` that takes
/// [`Json<In2>`] as the second argument and assigns [`StatusCode::INTERNAL_SERVER_ERROR`]
/// to any error result.
pub fn handler_fn2r<In1, In2, O, E, Fut, S>(
    f: impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static,
) -> impl Fn(
    In1,
    Json<In2>,
) -> Pin<Box<(dyn Future<Output = (StatusCode, Json<Result<O, E>>)> + Send + 'static)>>
       + Send
       + 'static
       + Clone
where
    In1: FromRequestParts<S> + Send + 'static,
    In2: DeserializeOwned + Send + 'static,
    Result<O, E>: Serialize,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<O, E>> + Send,
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

/// Returns a handler for `Fn(In1, In2) -> Future<Output = Result<O, (StatusCode, E)>` that takes
/// [`Json<In2>`] as the second argument.
pub fn handler_fn2rs<In1, In2, O, E, Fut, S>(
    f: impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static,
) -> impl Fn(
    In1,
    Json<In2>,
) -> Pin<
    Box<(dyn Future<Output = Result<Json<O>, (StatusCode, Json<E>)>> + Send + 'static)>,
>
       + Send
       + 'static
       + Clone
where
    In1: FromRequestParts<S> + Send + 'static,
    In2: DeserializeOwned + Send + 'static,
    O: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<O, (StatusCode, E)>> + Send,
{
    move |req_part, Json(input)| {
        let f = f.clone();
        Box::pin(async move {
            let out = f(req_part, input).await;
            match out {
                Ok(out) => Ok(Json(out)),
                Err((status_code, err)) => Err((status_code, Json(err))),
            }
        })
    }
}

#[cfg(test)]
fn _typecheck_handler_fn2r<In1, In2, O, E, Fut, S>(
    f: impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static,
) where
    In1: FromRequestParts<S> + Send + 'static,
    In2: DeserializeOwned + Send + 'static,
    O: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<O, E>> + Send,
{
    use super::_axum_handler_type_checker_2_args_generic;

    _axum_handler_type_checker_2_args_generic::<_, Json<In2>, _, _, S>(&handler_fn2r(f));
}

//=================
// Handlers for AsyncFn[x]

/// Returns a handler for `AsyncFn2<Out = Result<O, E>>` that takes
/// [`Json<F::In2>`] as the second argument and assigns [`StatusCode::INTERNAL_SERVER_ERROR`]
/// to any error result.
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

/// Returns a handler for `AsyncFn2<Out = Result<O, E>>` that takes
/// [`Json<F::In2>`] as the second argument and
/// ????
pub fn handler_asyncfn2rs<O, E, F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<
    Box<(dyn Future<Output = Result<Json<O>, (StatusCode, Json<E>)>> + Send + 'static)>,
>
       + Send
       + 'static
       + Clone
where
    F: AsyncFn2<Out = Result<O, (StatusCode, E)>> + Send + Sync + Clone + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    O: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
{
    handler_fn2rs(f.into_fnonce_when_clone())
}

/// Type wrapper for `AsyncFn2<Out = Result<O, E>>` that creates a handler that takes
/// [`Json<F::In2>`] as the second argument and assigns [`StatusCode::INTERNAL_SERVER_ERROR`]
/// to any error result. More convenient to use than [`handler_asyncfn2r`].
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

/// Type wrapper for `AsyncFn2<Out = Result<O, E>>` that creates a handler that takes
/// [`Json<F::In2>`] as the second argument and assigns [`StatusCode::INTERNAL_SERVER_ERROR`]
/// to any error result. More convenient to use than [`handler_asyncfn2r`].
#[derive(Clone)]
pub struct HandlerAsyncFn2rs<F>(pub F);

pub type HandlerAsyncFn2rsArc<F> = HandlerAsyncFn2rs<Arc<F>>;

impl<F> HandlerAsyncFn2rsArc<F> {
    pub fn new(f: F) -> Self {
        HandlerAsyncFn2rs(f.into())
    }
}

impl<O, E, F, S> Handler<(), S> for HandlerAsyncFn2rs<F>
where
    F: AsyncFn2<Out = Result<O, (StatusCode, E)>> + Send + Sync + Clone + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    O: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, req: Request, state: S) -> Self::Future {
        handler_asyncfn2rs::<O, E, F, S>(self.0).call(req, state)
    }
}

/// JSON handler type wrapper for `AsyncFn2<Out = Result<O, E>>` and a function that maps errors
/// to a pair [`(StatusCode, EMO)`].
pub struct HandlerAsyncFn2rWithErrorMapper<EMI, EMO, F, M>(pub F, M, PhantomData<(EMI, EMO)>);

impl<F, M, EMI, EMO> HandlerAsyncFn2rWithErrorMapper<EMI, EMO, F, M> {
    pub fn new(f: F, m: M) -> Self {
        Self(f, m, PhantomData)
    }
}

impl<EMI, EMO, F, M> Clone for HandlerAsyncFn2rWithErrorMapper<EMI, EMO, F, M>
where
    F: Clone,
    M: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<O, E, EMI, EMO, F, M, S> Handler<(), S> for HandlerAsyncFn2rWithErrorMapper<EMI, EMO, F, M>
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static + Clone,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    O: Serialize + Send,
    E: Serialize + Into<EMI> + Send,
    S: Send + Sync + 'static,
    M: Fn(EMI) -> (StatusCode, EMO) + Send + Sync + 'static + Clone,
    EMI: Send + 'static + Sync,
    EMO: Serialize + Send + 'static + Sync,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, req: Request, state: S) -> Self::Future {
        let mf = WithMappedErrors::new(self.0, self.1);
        let h = HandlerAsyncFn2rs(mf);
        h.call(req, state)
    }
}

/// JSON handler type wrapper for `AsyncFn2<Out = Result<O, E>>` and a function that maps errors
/// to a pair [`(StatusCode, EMO)`].
pub struct HandlerAsyncFn2rWithErrorMapperOld<EMI, EMO, F, M>(pub F, M, PhantomData<(EMI, EMO)>);

impl<F, M, EMI, EMO> HandlerAsyncFn2rWithErrorMapperOld<EMI, EMO, F, M> {
    pub fn new(f: F, m: M) -> Self {
        Self(f, m, PhantomData)
    }
}

impl<EMI, EMO, F, M> Clone for HandlerAsyncFn2rWithErrorMapperOld<EMI, EMO, F, M>
where
    F: Clone,
    M: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<O, E, EMI, EMO, F, M, S> Handler<(), S> for HandlerAsyncFn2rWithErrorMapperOld<EMI, EMO, F, M>
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static + Clone,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    O: Serialize + Send,
    E: Serialize + Into<EMI> + Send,
    S: Send + Sync + 'static,
    M: Fn(EMI) -> (StatusCode, EMO) + Send + Sync + 'static + Clone,
    EMI: Send + 'static + Sync,
    EMO: Serialize + Send + 'static + Sync,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, req: Request, state: S) -> Self::Future {
        let mf = WithMappedErrorsOld(self.0, self.1, PhantomData);
        let h = mf.into_fnonce_when_clone();
        h.call(req, state)
    }
}

/// Wrapper type that takes an `AsyncFn2<Out = Result<O, E>>` and a function that maps errors
/// to a pair [`(StatusCode, EMO)`], producing an
/// [`AsyncFn2<In1 = F::In1, In2 = Json<F::In2>, Out = Result<Json<O>, (StatusCode, Json<EMO>)>>`].
struct WithMappedErrorsOld<EMI, EMO, F, M>(F, M, PhantomData<(EMI, EMO)>);

impl<EMI, EMO, F: Clone, M: Clone> Clone for WithMappedErrorsOld<EMI, EMO, F, M> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<O, E, EMI, EMO, F, M> AsyncFn2 for WithMappedErrorsOld<EMI, EMO, F, M>
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static,
    F::In2: DeserializeOwned,
    O: Serialize + Send,
    E: Serialize + Into<EMI> + Send,
    M: Fn(EMI) -> (StatusCode, EMO) + Send + Sync + 'static,
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
