use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use foa::fun::AsyncFn2;
use foa::web::axum::handler_asyncfn2r;
use serde::{de::DeserializeOwned, Serialize};
use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

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
        ) -> impl FnOnce(
            F::In1,
            Json<F::In2>,
        ) -> Pin<
            Box<(dyn Future<Output = (StatusCode, Json<F::Out>)> + Send + 'static)>,
        >
               + Send
               + Sync // not needed for Axum
               + 'static
               + Clone {
            handler_asyncfn2r::<O, E, F, S>(self.0)
        }
    }
}
