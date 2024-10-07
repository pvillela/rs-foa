use super::ctx::Ctx;
use crate::svc::{FooIn, FooOut, FooSflI};
use foa::{
    db::sqlx::AsyncTxFn,
    fun::AsyncFn2,
    tokio::task_local::{invoke_tl_scoped, tl_scoped, TaskLocal, TaskLocalCtx},
    Result,
};
use std::{future::Future, pin::Pin};

type CtxTl = <Ctx as TaskLocalCtx>::TaskLocal;
type CtxTlValue = <CtxTl as TaskLocal>::Value;

pub struct FooSflIC;

impl AsyncFn2 for FooSflIC {
    type In1 = CtxTlValue;
    type In2 = FooIn;
    type Out = Result<FooOut>;

    async fn invoke(&self, input1: Self::In1, input2: Self::In2) -> Self::Out {
        invoke_tl_scoped::<_, CtxTl>(&FooSflI(Ctx).in_tx(), input1, input2).await
    }
}

/// This requires [`Ctx`] : [`Clone`]
pub fn make_foo_sfl(
) -> impl FnOnce(CtxTlValue, FooIn) -> Pin<Box<(dyn Future<Output = Result<FooOut>> + Send + 'static)>>
       + Send
       + Sync // optional, results from Self: Sync
       + Clone
       + 'static {
    tl_scoped::<_, CtxTl>(FooSflI(Ctx).in_tx()).into_fnonce_with_arc()
}
