use crate::context::ErrCtx;
use crate::error::{ErrorKind, JsonBoxError};
use crate::fun::AsyncFn2;
use crate::Error;
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::any::Any;
use std::error::Error as _;
use std::future::Future;
use std::marker::PhantomData;

/// Checks a closure for compliance with Axum Handler impl requirements.
pub(crate) fn _axum_handler_type_checker_2_args_generic<In1, In2, Out, Fut, S>(
    _f: &(impl FnOnce(In1, In2) -> Fut + Clone + Send + 'static),
) where
    Fut: Future<Output = Out> + Send,
    In1: FromRequestParts<S> + Send,
    In2: FromRequest<S> + Send,
    Out: IntoResponse,
    S: Send + Sync + 'static,
{
}

/// Wrapper type that takes an `AsyncFn2<Out = Result<O, E>>` and a function that maps errors
/// to a pair [`(StatusCode, EMO)`], producing an
/// ????
struct WithMappedErrors0<In2, EMI, EMO, F, M, S>(F, M, PhantomData<(In2, EMI, EMO, S)>);

impl<In2, EMI, EMO, F: Clone, M: Clone, S> Clone for WithMappedErrors0<In2, EMI, EMO, F, M, S> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<In2, O, E, EMI, EMO, F, M, S> AsyncFn2 for WithMappedErrors0<In2, EMI, EMO, F, M, S>
where
    F: AsyncFn2<Out = Result<O, E>> + Send + Sync + 'static,
    In2: FromRequest<S> + Send + Sync,
    F::In2: From<In2>,
    O: Serialize + Send,
    E: Serialize + Into<EMI> + Send,
    M: Fn(EMI) -> (StatusCode, EMO) + Send + Sync + 'static,
    EMI: Send + Sync,
    EMO: Send + Sync,
    S: Send + Sync + 'static,
{
    type In1 = F::In1;
    type In2 = In2;
    type Out = Result<Json<O>, (StatusCode, Json<EMO>)>;

    async fn invoke(&self, in1: Self::In1, in2: Self::In2) -> Self::Out {
        let out_f = self.0.invoke(in1, in2.into()).await;
        match out_f {
            Ok(out) => Ok(Json(out)),
            Err(err) => {
                let (status_code, err) = self.1(err.into());
                Err((status_code, Json(err)))
            }
        }
    }
}

/// Wrapper type that takes an `AsyncFn2<Out = Result<O, E>>` and a function that maps errors
/// to a pair [`(StatusCode, EMO)`], producing an
/// ????
pub struct WithMappedErrors<EMI, EMO, F, M>(F, M, PhantomData<(EMI, EMO)>);

impl<EMI, EMO, F, M> WithMappedErrors<EMI, EMO, F, M> {
    pub fn new(f: F, m: M) -> Self {
        Self(f, m, PhantomData)
    }
}

impl<EMI, EMO, F: Clone, M: Clone> Clone for WithMappedErrors<EMI, EMO, F, M> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<O, E, EMI, EMO, F, M> AsyncFn2 for WithMappedErrors<EMI, EMO, F, M>
where
    F: AsyncFn2<Out = Result<O, E>> + Sync,
    O: Send,
    E: Into<EMI>,
    M: Fn(EMI) -> (StatusCode, EMO) + Sync,
    EMI: Sync,
    EMO: Send + Sync,
{
    type In1 = F::In1;
    type In2 = F::In2;
    type Out = Result<O, (StatusCode, EMO)>;

    async fn invoke(&self, in1: Self::In1, in2: Self::In2) -> Self::Out {
        let out_f = self.0.invoke(in1, in2.into()).await;
        match out_f {
            Ok(out) => Ok(out),
            Err(err) => Err(self.1(err.into())),
        }
    }
}

pub fn default_mapper<CTX: ErrCtx>(be: JsonBoxError) -> (StatusCode, JsonBoxError) {
    // let e_opt = be.downcast_ref::<Error<CTX>>();
    // if e_opt.is_none() {
    //     return (StatusCode::INTERNAL_SERVER_ERROR, be);
    // }

    // let e = e_opt.unwrap();

    // let src = e.source();
    // if src.is_none() {
    //     return (StatusCode::BAD_REQUEST, be);
    // }

    // let e1 = src.unwrap();

    // let e1d = e1.downcast_ref::<sqlx::Error>();
    // if e1d.is_none() {
    //     // return (StatusCode::BAD_REQUEST, be);
    // }

    // let e2 = e1d.unwrap();

    // let x = e2.to_string();
    // let err = FOO_ERROR.new_error_with_args::<()>([&x]);
    // let berr: Box<dyn std::error::Error> = Box::new(err);
    // return (StatusCode::INTERNAL_SERVER_ERROR, berr);

    const FOO_ERROR: ErrorKind<1, false> =
        ErrorKind::new("FOO_ERROR", "foo error {foo}", ["foo"], None);

    let be_any = &be.0 as &dyn Any;
    let ret = match be_any.downcast_ref::<Error<CTX>>() {
        Some(e) => {
            let src = e.source();
            match src {
                None => (StatusCode::INTERNAL_SERVER_ERROR, be),
                Some(e1) => match e1.downcast_ref::<sqlx::Error>() {
                    None => (StatusCode::BAD_REQUEST, be),
                    Some(e2) => {
                        let x = e2.to_string();
                        let err = FOO_ERROR.new_error_with_args::<()>([&x]);
                        let berr = JsonBoxError::new(err);
                        (StatusCode::INTERNAL_SERVER_ERROR, berr)
                    }
                },
            }
        }
        None => (StatusCode::INTERNAL_SERVER_ERROR, be),
    };
    ret
}
