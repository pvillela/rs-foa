use crate::foa_old::fun::async_rfn::{AsyncRFn, AsyncRFn2};
use foa::tokio::task_local::TaskLocal;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct TlScopedOld<F, TL>(F, PhantomData<TL>);

impl<F, TL> TlScopedOld<F, TL> {
    pub fn new(f: F) -> Self {
        TlScopedOld(f, PhantomData)
    }
}

impl<F, TL> AsyncRFn2 for TlScopedOld<F, TL>
where
    TL: TaskLocal + Sync,
    TL::Value: Send,
    F: AsyncRFn + Sync,
{
    type In1 = TL::Value;
    type In2 = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, value: Self::In1, input: Self::In2) -> Result<Self::Out, Self::E> {
        let lk = TL::local_key();
        lk.scope(value, self.0.invoke(input)).await
    }
}

pub fn tl_scoped_old<'a, F, TL>(
    f: F,
) -> impl AsyncRFn2<In1 = TL::Value, In2 = F::In, Out = F::Out, E = F::E> + 'a
where
    TL: TaskLocal + Sync + 'a,
    TL::Value: Send,
    F: AsyncRFn + Sync + 'a,
{
    TlScopedOld::<_, TL>::new(f)
}

pub async fn invoke_tl_scoped_old<F, TL>(f: &F, in1: TL::Value, in2: F::In) -> Result<F::Out, F::E>
where
    TL: TaskLocal + Sync,
    TL::Value: Send,
    F: AsyncRFn + Sync,
{
    tl_scoped_old::<_, TL>(f).invoke(in1, in2).await
}
