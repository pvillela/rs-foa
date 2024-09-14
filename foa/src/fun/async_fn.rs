use std::{future::Future, pin::Pin, sync::Arc};

pub trait AsyncFn {
    type In: Send;
    type Out: Send;

    fn invoke(&self, input: Self::In) -> impl Future<Output = Self::Out> + Send;
}

impl<F: AsyncFn> AsyncFn for &F {
    type In = F::In;
    type Out = F::Out;

    fn invoke(&self, input: Self::In) -> impl Future<Output = Self::Out> + Send {
        F::invoke(self, input)
    }
}

impl<F> AsyncFn for Arc<F>
where
    F: AsyncFn + Sync + Send,
{
    type In = F::In;
    type Out = F::Out;

    fn invoke(&self, input: Self::In) -> impl Future<Output = Self::Out> + Send {
        async { self.as_ref().invoke(input).await }
    }
}

pub trait AsyncFn2 {
    type In1: Send;
    type In2: Send;
    type Out: Send;

    fn invoke(&self, in1: Self::In1, in2: Self::In2) -> impl Future<Output = Self::Out> + Send;

    /// Reifies `self` as an `async Fn`
    fn into_fn<'a>(
        self,
    ) -> impl Fn(Self::In1, Self::In2) -> Pin<Box<(dyn Future<Output = Self::Out> + Send + 'a)>>
           + Send
           + Sync // optional, results from Self: Sync
           + 'a
           + Clone
    where
        Self: Send
            + Sync // optional if resulting Fn doesn't have to be Sync
            + Clone
            + 'a,
    {
        move |in1, in2| {
            let f = self.clone();
            Box::pin(async move { f.invoke(in1, in2).await })
        }
    }
}

impl<F> AsyncFn2 for Arc<F>
where
    F: AsyncFn2 + Sync + Send,
{
    type In1 = F::In1;
    type In2 = F::In2;
    type Out = F::Out;

    fn invoke(&self, in1: Self::In1, in2: Self::In2) -> impl Future<Output = Self::Out> + Send {
        async { self.as_ref().invoke(in1, in2).await }
    }
}

impl<F: AsyncFn2> AsyncFn2 for &F {
    type In1 = F::In1;
    type In2 = F::In2;
    type Out = F::Out;

    fn invoke(&self, in1: Self::In1, in2: Self::In2) -> impl Future<Output = Self::Out> + Send {
        F::invoke(self, in1, in2)
    }
}
