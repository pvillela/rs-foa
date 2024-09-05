use std::{future::Future, marker::PhantomData};

pub trait AsyncFn {
    type In;
    type Out;

    #[allow(async_fn_in_trait)]
    async fn invoke(input: Self::In) -> Self::Out;
}

pub trait AsyncRFn {
    type In: Send;
    type Out: Send;
    type E;

    #[allow(async_fn_in_trait)]
    fn invoke(input: Self::In) -> impl Future<Output = Result<Self::Out, Self::E>> + Send;
}

pub struct AsyncRFnAsAsyncFn<F: AsyncRFn>(PhantomData<F>);

impl<F> AsyncFn for AsyncRFnAsAsyncFn<F>
where
    F: AsyncRFn,
{
    type In = F::In;
    type Out = Result<F::Out, F::E>;

    async fn invoke(input: Self::In) -> Self::Out {
        F::invoke(input).await
    }
}
