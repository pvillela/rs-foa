use crate::fun::async_rfn::{AsyncRFn, AsyncRFn2};
use std::{future::Future, pin::Pin, sync::Arc};

pub trait AsyncFn {
    type In: Send;
    type Out: Send;

    fn invoke(&self, input: Self::In) -> impl Future<Output = Self::Out> + Send;

    fn into_asyncrfn<O, E>(self) -> impl AsyncRFn<In = Self::In, Out = O, E = E>
    where
        Self: AsyncFn<Out = Result<O, E>> + Sized,
        O: Send,
        E: Send,
    {
        WAsyncFn(self)
    }
}
struct WAsyncFn<F>(F);

impl<O, E, F> AsyncRFn for WAsyncFn<F>
where
    F: AsyncFn<Out = Result<O, E>>,
    O: Send,
    E: Send,
{
    type In = F::In;
    type Out = O;
    type E = E;

    fn invoke(&self, input: Self::In) -> impl Future<Output = Result<O, E>> + Send {
        self.0.invoke(input)
    }
}

#[derive(Clone)]
struct WAsyncFn2<F>(F);

impl<O, E, F> AsyncRFn2 for WAsyncFn2<F>
where
    F: AsyncFn2<Out = Result<O, E>>,
    O: Send,
    E: Send,
{
    type In1 = F::In1;
    type In2 = F::In2;
    type Out = O;
    type E = E;

    fn invoke(&self, in1: Self::In1, in2: Self::In2) -> impl Future<Output = Result<O, E>> + Send {
        self.0.invoke(in1, in2)
    }
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

    fn into_asyncrfn2<O, E>(
        self,
    ) -> impl AsyncRFn2<In1 = Self::In1, In2 = Self::In2, Out = O, E = E> + Send + Sync + 'static
    where
        Self: AsyncFn2<Out = Result<O, E>> + Sized + Send + Sync + 'static,
        O: Send,
        E: Send,
    {
        WAsyncFn2(self)
    }

    fn into_asyncrfn2_when_clone<O, E>(
        self,
    ) -> impl AsyncRFn2<In1 = Self::In1, In2 = Self::In2, Out = O, E = E> + Send + Sync + 'static + Clone
    where
        Self: AsyncFn2<Out = Result<O, E>> + Sized + Send + Sync + 'static + Clone,
        O: Send,
        E: Send,
    {
        WAsyncFn2(self)
    }

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
