use crate::{fun::AsyncFn2, web::WithMappedErrors};
use axum::{
    extract::{FromRequestParts, Request},
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

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

/// Returns a handler for `Fn(In1, In2) -> Future<Output = Result<O, (StatusCode, E)>` that takes
/// [`Json<In2>`] as the second argument and returns [`Result<Json<O>, (StatusCode, Json<E>)>`].
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
fn _typecheck_handler_fn2rs<In1, In2, O, E, Fut, S>(
    f: impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static,
) where
    In1: FromRequestParts<S> + Send + 'static,
    In2: DeserializeOwned + Send + 'static,
    O: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<O, (StatusCode, E)>> + Send,
{
    use super::_axum_handler_type_checker_2_args_generic;

    _axum_handler_type_checker_2_args_generic::<_, Json<In2>, _, _, S>(&handler_fn2rs(f));
}

/// Returns a handler for [`AsyncFn2<Out = Result<O, (StatusCode, E)>>`] that takes
/// [`Json<F::In2>`] as the second argument and returns [`Result<Json<O>, (StatusCode, Json<E>)>`].
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
    // could have used `f.into_fnonce_when_clone()` but that would involve another box-pinning
    let fc = move |in1, in2| async move { f.invoke(in1, in2).await };
    handler_fn2rs(fc)
}

/// Returns a handler for the [`Arc`] of an [`AsyncFn2<Out = Result<O, (StatusCode, E)>>`] that takes
/// [`Json<F::In2>`] as the second argument and returns [`Result<Json<O>, (StatusCode, Json<E>)>`].
pub fn handler_asyncfn2rs_arc<O, E, F, S>(
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
    F: AsyncFn2<Out = Result<O, (StatusCode, E)>> + Send + Sync + 'static,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    O: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
{
    handler_asyncfn2rs(Arc::new(f))
}

/// Type wrapper for `AsyncFn2<Out = Result<O, (StatusCode, E)>>` that implements a handler that takes
/// [`Json<F::In2>`] as the second argument and returns [`Result<Json<O>, (StatusCode, Json<E>)>`].
/// More convenient to use than [`handler_asyncfn2rs`] due to better type inference for type constructors
/// than for functions.
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

/// Type wrapper for an `AsyncFn2<Out = Result<O, E>>` and a function that maps errors
/// to a pair [`(StatusCode, EMO)`]; implements a handler that takes
/// [`Json<F::In2>`] as the second argument and returns [`Result<Json<O>, (StatusCode, Json<E>)>`].
pub struct HandlerAsyncFn2rsWithErrorMapper<EMI, EMO, F, M>(F, M, PhantomData<(EMI, EMO)>);

impl<O, E, EMI, EMO, F, M> HandlerAsyncFn2rsWithErrorMapper<EMI, EMO, F, M>
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static + Clone,
    F::In2: DeserializeOwned,
    O: Serialize + Send,
    E: Into<EMI> + Send,
    M: Fn(EMI) -> (StatusCode, EMO) + Send + Sync + 'static + Clone,
    EMI: Send + 'static + Sync,
    EMO: Serialize + Send + 'static + Sync,
{
    pub fn new(f: F, m: M) -> Self {
        Self(f, m, PhantomData)
    }
}

impl<EMI, EMO, F, M> Clone for HandlerAsyncFn2rsWithErrorMapper<EMI, EMO, F, M>
where
    F: Clone,
    M: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<O, E, EMI, EMO, F, M, S> Handler<(), S> for HandlerAsyncFn2rsWithErrorMapper<EMI, EMO, F, M>
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static + Clone,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    O: Serialize + Send,
    E: Into<EMI> + Send,
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
