use crate::foa_exp::fun::async_rfn::{AsyncRFn, AsyncRFn2};
use crate::foa_exp::tokio::task_local_old::tl_scoped_old;
use foa::db::sqlx::{AsyncTxFn, Db};
use foa::tokio::task_local::{invoke_tl_scoped, TaskLocal};

#[deprecated]
struct InTxOld<F>(F);

#[allow(deprecated)]
impl<F> AsyncRFn for InTxOld<F>
where
    F: AsyncTxFn + Sync,
{
    type In = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, input: Self::In) -> Result<Self::Out, Self::E> {
        let pool = F::Db::pool().await?;
        let mut tx = pool.begin().await?;
        let output = self.0.invoke(input, &mut tx).await?;
        tx.commit().await?;
        Ok(output)
    }
}

#[deprecated]
#[allow(deprecated)]
pub fn in_tx_old<'a, F>(f: F) -> impl AsyncRFn<In = F::In, Out = F::Out, E = F::E> + 'a
where
    F: AsyncTxFn + Sync + Send + 'a,
{
    InTxOld(f)
}

#[deprecated]
#[allow(deprecated)]
pub async fn invoke_in_tx_old<F>(f: &F, input: F::In) -> Result<F::Out, F::E>
where
    F: AsyncTxFn + Sync,
{
    AsyncRFn::invoke(&InTxOld(f), input).await
}

#[deprecated]
#[allow(deprecated)]
pub fn in_tx_tl_scoped_old<'a, F, TL>(
    f: F,
) -> impl AsyncRFn2<In1 = TL::Value, In2 = F::In, Out = F::Out, E = F::E> + 'a
where
    TL: TaskLocal + Sync + 'static,
    TL::Value: Send,
    F: AsyncTxFn + Sync + Send + 'a,
{
    let wf1 = in_tx_old(f);
    tl_scoped_old::<_, TL>(wf1)
}

#[deprecated]
pub async fn invoke_in_tx_tl_scoped_old<F, TL>(
    f: &F,
    in1: TL::Value,
    in2: F::In,
) -> Result<F::Out, F::E>
where
    TL: TaskLocal + Sync + 'static,
    TL::Value: Send,
    F: AsyncTxFn + Sync,
{
    let wf1 = f.in_tx();
    invoke_tl_scoped::<_, TL>(&wf1, in1, in2).await
}
