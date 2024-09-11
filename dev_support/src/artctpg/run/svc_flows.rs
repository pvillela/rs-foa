use super::ctx::Ctx;
use crate::artctpg::svc::{FooIn, FooOut, FooSflI};
use foa::{
    db::sqlx::AsyncTxFn,
    error::FoaError,
    fun::AsyncRFn2,
    tokio::task_local::{TaskLocal, TaskLocalCtx},
};
use std::{future::Future, pin::Pin, sync::Arc};

type CtxTl = <Ctx as TaskLocalCtx>::TaskLocal;
type CtxTlValue = <CtxTl as TaskLocal>::Value;

pub struct FooSflIC;

impl AsyncRFn2 for FooSflIC {
    type In1 = CtxTlValue;
    type In2 = FooIn;
    type Out = FooOut;
    type E = FoaError<Ctx>;

    async fn invoke(&self, input1: Self::In1, input2: Self::In2) -> Result<Self::Out, Self::E> {
        FooSflI(Ctx)
            .invoke_in_tx_tl_scoped::<CtxTl>(input1, input2)
            .await
    }
}

/// This requires [`Ctx`] : [`Clone`]
pub fn make_foo_sfl() -> impl FnOnce(
    CtxTlValue,
    FooIn,
) -> Pin<Box<(dyn Future<Output = Result<FooOut, FoaError<Ctx>>> + Send + 'static)>>
       + Send
       + Sync // optional, results from Self: Sync
       + Clone
       + 'static {
    Arc::new(FooSflI(Ctx).in_tx_tl_scoped::<CtxTl>()).into_fn()
}
