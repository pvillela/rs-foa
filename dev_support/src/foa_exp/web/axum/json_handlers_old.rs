use crate::foa_exp::fun::async_rfn::{AsyncRFn, AsyncRFn2};
use axum::extract::{FromRequestParts, Request};
use axum::handler::Handler;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use foa::context::Source;
use foa::db::sqlx::{in_tx, AsyncTxFn};
use foa::fun::AsyncFn2;
use foa::tokio::task_local::{invoke_tl_scoped, TaskLocal};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// region:      --- AsyncFn2rHandler

/// Returns a handler for `Fn(In1, In2) -> Future<Output = Result<O, E>` that takes
/// [`Json<In2>`] as the second argument, returns `(StatusCode, Json<Result<O, E>>)`,
/// and assigns [`StatusCode::INTERNAL_SERVER_ERROR`] to any error result.
pub fn handler_fn2r<In1, In2, O, E, Fut, S>(
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
    Result<O, E>: Serialize,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<O, E>> + Send,
{
    move |req_part, Json(input)| {
        let f = f.clone();
        Box::pin(async move {
            let out = f(req_part, input).await;
            match out {
                Ok(out) => Ok(Json(out)),
                Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err))),
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
    use foa::web::axum::_axum_handler_type_checker_2_args_generic;

    _axum_handler_type_checker_2_args_generic::<_, Json<In2>, _, _, S>(&handler_fn2r(f));
}

/// Returns a handler for [`AsyncFn2<Out = Result<O, E>>`] that takes
/// [`Json<F::In2>`] as the second argument, returns [`Result<Json<O>, (StatusCode, Json<E>)>`],
/// and assigns [`StatusCode::INTERNAL_SERVER_ERROR`] to any error result.
pub fn handler_asyncfn2r<O, E, F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<
    Box<(dyn Future<Output = Result<Json<O>, (StatusCode, Json<E>)>> + Send + 'static)>,
>
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
    // could have used `f.into_fnonce_when_clone()` but that would involve another box-pinning
    let fc = move |in1, in2| async move { f.invoke(in1, in2).await };
    handler_fn2r(fc)
}

/// Returns a handler for the [`Arc`] of an [`AsyncFn2<Out = Result<O, E>>`] that takes
/// [`Json<F::In2>`] as the second argument, returns [`Result<Json<O>, (StatusCode, Json<E>)>`],
/// and assigns [`StatusCode::INTERNAL_SERVER_ERROR`] to any error result.
pub fn handler_asyncfn2r_arc<O, E, F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<
    Box<(dyn Future<Output = Result<Json<O>, (StatusCode, Json<E>)>> + Send + 'static)>,
>
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

/// Type wrapper for `AsyncFn2<Out = Result<O, E>>` that implements a handler that takes
/// [`Json<F::In2>`] as the second argument and assigns [`StatusCode::INTERNAL_SERVER_ERROR`]
/// to any error result.
/// More convenient to use than [`handler_asyncfn2r`] due to better type inference for type constructors
/// than for functions.
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

// endregion:   --- AsyncFn2rHandler

// region:      --- Type checker

#[cfg(test)]
use axum::{extract::FromRequest, response::IntoResponse};

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

// endregion:   --- Type checker

// region:      --- Handlers for AsyncRFn[x]

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
    Box<(dyn Future<Output = Result<Json<F::Out>, (StatusCode, Json<F::E>)>> + Send + 'static)>,
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
    handler_asyncfn2r(f.into_asyncfn2_when_clone())
}

pub fn handler_asyncrfn2_arc<F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<
    Box<(dyn Future<Output = Result<Json<F::Out>, (StatusCode, Json<F::E>)>> + Send + 'static)>,
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

// endregion:   --- Handlers for AsyncRFn[x]

// region:      --- deprecated handlers

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
    MF: Source<F> + 'static,
{
    let f = MF::source();
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
    MF: Source<F> + 'static,
{
    _axum_handler_type_checker_2_args_generic::<_, Json<F::In>, _, _, ()>(
        &handler_tx_headers_old::<F, MF, TL>,
    );
}

// endregion:   --- deprecated handlers
