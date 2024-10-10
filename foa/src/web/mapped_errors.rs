use crate::fun::AsyncFn2;
use http::StatusCode;
use std::marker::PhantomData;

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
