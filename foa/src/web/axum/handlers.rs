use crate::error::{ErrorExp, JserBoxError, VALIDATION_TAG};
use crate::fun::AsyncFn2;
use crate::Error;
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::future::Future;
use std::marker::PhantomData;
use valid::ValidationError;

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

pub fn default_mapper(err: Error) -> (StatusCode, JserBoxError) {
    match err.tag() {
        Some(&VALIDATION_TAG) => {
            let status_code = StatusCode::BAD_REQUEST;
            let err_exp_res: Result<ErrorExp<ValidationError>, Error> = err.into();
            match err_exp_res {
                Ok(ee) => (status_code, JserBoxError::new(ee)),
                Err(e) => (status_code, e.into()),
            }
        }
        _ => {
            err.backtrace()
                .map(|b| log::error!("error={err:?}, backtrace={b}"));
            (StatusCode::INTERNAL_SERVER_ERROR, err.into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::{ErrorWithNull, TrivialError};
    use std::mem::replace;

    // fn foo<T: std::error::Error + Send + Sync + 'static + Serialize>(
    //     mut e: JserBoxError,
    //     t1: T,
    // ) -> Option<T> {
    //     let x = &mut e.0;
    //     let z = x.as_any_mut();
    //     let dz = z.downcast_mut::<T>();
    //     let Some(t_ref) = dz else {
    //         return None;
    //     };
    //     let t = replace(t_ref, t1);
    //     Some(t)
    // }

    fn bar<T: std::error::Error + Send + Sync + 'static + Serialize>(
        mut e: JserBoxError,
    ) -> Option<T> {
        let x = &mut e.0;
        let z = x.as_any_mut();
        let dz = z.downcast_mut::<ErrorWithNull<T>>();
        let Some(t_ref) = dz else {
            return None;
        };
        let t = replace(t_ref, ErrorWithNull::Null);
        t.real()
    }

    // #[test]
    // fn test_foo() {
    //     let e1 = TrivialError("e1");
    //     let e3 = TrivialError("e3");

    //     let jsb_e3 = JserBoxError::new(e3);

    //     let e3a = foo(jsb_e3, e1).unwrap();
    //     assert_eq!(e3a.to_string(), "e3".to_owned());
    // }

    #[test]
    fn test_bar() {
        let e3 = TrivialError("e3");

        let jsb_e3 = JserBoxError::new(e3);

        let e3a: TrivialError = bar(jsb_e3).unwrap();
        assert_eq!(e3a.to_string(), "e3".to_owned());
    }
}
