use crate::fun::async_rfn::{AsyncRFn, AsyncRFn2};
use crate::tokio::task_local::{TaskLocal, TlScoped};

pub fn tl_scoped_old<'a, F, TL>(
    f: F,
) -> impl AsyncRFn2<In1 = TL::Value, In2 = F::In, Out = F::Out, E = F::E> + 'a
where
    TL: TaskLocal + Sync + 'a,
    TL::Value: Send,
    F: AsyncRFn + Sync + 'a,
{
    TlScoped::<_, TL>::new(f)
}

pub async fn invoke_tl_scoped_old<F, TL>(f: &F, in1: TL::Value, in2: F::In) -> Result<F::Out, F::E>
where
    TL: TaskLocal + Sync,
    TL::Value: Send,
    F: AsyncRFn + Sync,
{
    tl_scoped_old::<_, TL>(f).invoke(in1, in2).await
}
