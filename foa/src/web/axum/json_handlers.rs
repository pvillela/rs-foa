use crate::{
    context::LocaleSelf,
    db::sqlx::{in_tx, AsyncTxFn},
    fun::{AsyncFn2, AsyncRFn, AsyncRFn2},
    tokio::task_local::{invoke_tl_scoped, TaskLocal},
    trait_utils::Make,
};
use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    handler::Handler,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{future::Future, pin::Pin, sync::Arc};

// #[cfg(test)]
// use axum::extract::FromRequest;

//=================
// Type checker

#[cfg(test)]
/// Checks a closure for compliance with Axum Handler impl requirements.
fn _axum_handler_type_checker_2_args_generic<In1, In2, Out, Fut, S>(
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
) -> impl FnOnce(
    In1,
    Json<In2>,
) -> Pin<Box<(dyn Future<Output = Result<Json<Out>, Json<E>>> + Send + 'static)>>
       + Send
       + 'static
       + Clone
where
    In1: FromRequestParts<S> + Send + 'static,
    In2: DeserializeOwned + Send + 'static,
    Out: Serialize,
    E: Serialize,
    S: Send + Sync + 'static,
    Fut: Future<Output = Result<Out, E>> + Send,
{
    move |req_part, Json(input)| {
        let f = f.clone();
        Box::pin(async move {
            let output = f(req_part, input).await?;
            Ok(Json(output))
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
    // handler_asyncrfn2(f.into_asyncrfn2_when_clone())
    move |req_part, Json(input)| {
        let f = f.clone();
        Box::pin(async move {
            let out = f.invoke(req_part, input).await;
            let status = match out {
                Ok(_) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(out))
        })
    }
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

pub mod from_scratch {
    use super::*;

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
            Box::pin(async move {
                // Split the request into parts and body
                let (mut parts, body) = req.into_parts();

                // Extract T1 from request parts
                let t1 = match F::In1::from_request_parts(&mut parts, &state).await {
                    Ok(value) => value,
                    Err(rejection) => return rejection.into_response(),
                };

                // Reconstruct the request
                let req = Request::from_parts(parts, body);

                // Extract T2 from the full request
                let Json(t2) = match Json::<F::In2>::from_request(req, &state).await {
                    Ok(value) => value,
                    Err(rejection) => return rejection.into_response(),
                };

                // Call the wrapped function with extracted values
                let out = self.0.invoke(t1, t2).await;

                // Convert the result to a response
                let status = match out {
                    Ok(_) => StatusCode::OK,
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (status, Json(out)).into_response()
            })
        }
    }
}

pub mod direct {
    use std::marker::PhantomData;

    use super::*;

    #[derive(Clone)]
    pub struct HandlerAsyncFn2r<F, S = ()>(pub F, PhantomData<S>);

    impl<O, E, F, S> HandlerAsyncFn2r<F, S>
    where
        F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static + Clone,
        F::In1: FromRequestParts<S>,
        F::In2: DeserializeOwned,
        O: Serialize,
        E: Serialize,
        S: Send + Sync + 'static,
    {
        pub fn new(f: F) -> Self {
            HandlerAsyncFn2r(f, PhantomData)
        }

        pub fn handler(
            self,
        ) -> impl Fn(
            F::In1,
            Json<F::In2>,
        )
            -> Pin<Box<(dyn Future<Output = (StatusCode, Json<F::Out>)> + Send + 'static)>>
               + Send
               + Sync // not needed for Axum
               + 'static
               + Clone {
            handler_asyncfn2r::<O, E, F, S>(self.0)
        }
    }
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

//=================
// Handlers for AsyncRFn[x]

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
    Box<(dyn Future<Output = (StatusCode, Json<Result<F::Out, F::E>>)> + Send + 'static)>,
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
    // move |req_part, Json(input)| {
    //     let f = f.clone();
    //     Box::pin(async move {
    //         let out = f.invoke(req_part, input).await;
    //         let status = match out {
    //             Ok(_) => StatusCode::OK,
    //             Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    //         };
    //         (status, Json(out))
    //     })
    // }
    handler_asyncfn2r(f.into_asyncfn2_when_clone())
}

pub fn handler_asyncrfn2_arc<F, S>(
    f: F,
) -> impl Fn(
    F::In1,
    Json<F::In2>,
) -> Pin<
    Box<(dyn Future<Output = (StatusCode, Json<Result<F::Out, F::E>>)> + Send + 'static)>,
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
    TL: TaskLocal<Value = Parts> + Sync + Send + 'static,
    F: AsyncTxFn + Sync + Send + 'static,
    F::In: DeserializeOwned,
    F::Out: Serialize,
    F::E: Serialize,
    MF: Make<F> + 'static,
{
    _axum_handler_type_checker_2_args_generic::<_, Json<F::In>, _, _, ()>(
        &handler_tx_headers_old::<F, MF, TL>,
    );
}
