use crate::{
    error::{self, Error, JserBoxError, VALIDATION_TAG},
    fun::AsyncFn2,
};
use http::StatusCode;
use log::{log, Level};
use std::marker::PhantomData;
use valid::ValidationError;

/// Wrapper type that takes an `AsyncFn2<Out = Result<O, E>>` and a function that maps errors
/// to a pair [`(StatusCode, EMO)`], producing an
/// `AsyncFn2<Out = Result<O, (StatusCode, EMO)>>`.
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

fn error_string_for_error_level(err: &Error) -> String {
    err.as_fmt().multi_speced_string([
        error::StringSpec::Dbg,
        error::StringSpec::Decor(
            &error::StringSpec::Recursive,
            Some("recursive_msg=("),
            Some(")"),
        ),
        error::StringSpec::Decor(&error::StringSpec::SourceDbg, Some("source="), None),
        error::StringSpec::Decor(&error::StringSpec::Backtrace, Some("backtrace=\n"), None),
    ])
}

pub fn default_mapper(err: Error) -> (StatusCode, JserBoxError) {
    match err.tag() {
        tag if tag == &VALIDATION_TAG => {
            let status_code = StatusCode::BAD_REQUEST;
            let err_exp_res = err.downcast_payload::<ValidationError>();
            match err_exp_res {
                Ok(ee) => (status_code, ee.into_sererror_with_payload([]).into()),
                Err(e) => (
                    status_code,
                    e.to_sererror_no_payload_src([
                        error::StringSpec::Dbg,
                        error::StringSpec::Recursive,
                    ])
                    .into(),
                ),
            }
        }
        _ => {
            log!(Level::Error, "{}", error_string_for_error_level(&err));
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                err.to_sererror_no_payload_src([
                    error::StringSpec::Dbg,
                    error::StringSpec::Recursive,
                ])
                .into(),
            )
        }
    }
}

#[cfg(test)]
/// For exploratory purpuses
pub fn default_mapper1(err: Error) -> (StatusCode, JserBoxError) {
    match err.tag() {
        tag if tag == &VALIDATION_TAG => {
            let status_code = StatusCode::BAD_REQUEST;
            err.chained_map(
                |e| {
                    e.with_downcast_payload::<ValidationError, _>(|ee| {
                        (status_code, ee.into_sererror_with_payload([]).into())
                    })
                },
                |e| {
                    (
                        status_code,
                        e.to_sererror_no_payload_src([
                            error::StringSpec::Dbg,
                            error::StringSpec::Recursive,
                        ])
                        .into(),
                    )
                },
            )
        }
        _ => {
            log!(Level::Error, "{}", error_string_for_error_level(&err));
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                err.to_sererror_no_payload_src([
                    error::StringSpec::Dbg,
                    error::StringSpec::Recursive,
                ])
                .into(),
            )
        }
    }
}
