use std::{future::Future, pin::Pin, sync::Arc};

pub trait AsyncFn {
    type In;
    type Out;

    #[allow(async_fn_in_trait)]
    async fn invoke(&self, input: Self::In) -> Self::Out;
}

pub trait AsyncRFn {
    type In: Send;
    type Out: Send;
    type E;

    fn invoke(&self, input: Self::In) -> impl Future<Output = Result<Self::Out, Self::E>> + Send;

    fn compose<'a, G>(self, g: G) -> impl AsyncRFn<In = G::In, Out = Self::Out, E = Self::E> + 'a
    where
        Self: AsyncRFn + Sync + Sized + 'a,
        G: AsyncRFn<Out = Self::In> + Sync + 'a,
        Self::E: From<G::E>,
        G::E: Send,
    {
        compose(self, g)
    }

    /// Reifies `self` as an `async Fn`
    fn into_fn(
        self,
    ) -> impl Fn(
        Self::In,
    ) -> Pin<Box<(dyn Future<Output = Result<Self::Out, Self::E>> + Send + 'static)>>
           + Send
           + Sync // optional, results from Self: Sync
           + 'static
           + Clone
    where
        Self: Send
            + Sync // optional if resulting Fn doesn't have to be Sync
            + Clone
            + 'static,
    {
        move |input| {
            let f = self.clone();
            Box::pin(async move {
                let output = f.invoke(input).await?;
                Ok(output)
            })
        }
    }
}

impl<F: AsyncRFn> AsyncRFn for &F {
    type In = F::In;
    type Out = F::Out;
    type E = F::E;

    fn invoke(&self, input: Self::In) -> impl Future<Output = Result<Self::Out, Self::E>> + Send {
        F::invoke(self, input)
    }
}

struct ARFnCompose<F, G>(F, G);
impl<F, G> AsyncRFn for ARFnCompose<F, G>
where
    F: AsyncRFn + Sync,
    G: AsyncRFn<Out = F::In> + Sync,
    F::E: From<G::E>,
    G::E: Send,
{
    type In = G::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, input: Self::In) -> Result<Self::Out, Self::E> {
        self.0.invoke(self.1.invoke(input).await?).await
    }
}

pub fn compose<'a, F, G>(f: F, g: G) -> impl AsyncRFn<In = G::In, Out = F::Out, E = F::E> + 'a
where
    F: AsyncRFn + Sync + 'a,
    G: AsyncRFn<Out = F::In> + Sync + 'a,
    F::E: From<G::E>,
    G::E: Send,
{
    ARFnCompose::<F, G>(f, g)
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fmt::Debug;

    async fn _foo<F, G>(f: F, g: G, input: G::In)
    where
        F: AsyncRFn + Sync,
        G: AsyncRFn<Out = F::In> + Sync,
        F::E: From<G::E> + Debug,
        G::In: Clone,
        G::E: Send,
        F::Out: Debug,
    {
        let h = compose(&f, &g);
        let res = h.invoke(input.clone()).await;
        println!("{res:?}");

        let h = (&f).compose(&g);
        let res = h.invoke(input).await;
        println!("{res:?}");
    }
}

impl<F> AsyncRFn for Arc<F>
where
    F: AsyncRFn + Sync + Send,
{
    type In = F::In;
    type Out = F::Out;
    type E = F::E;

    fn invoke(&self, input: Self::In) -> impl Future<Output = Result<Self::Out, Self::E>> + Send {
        async { self.as_ref().invoke(input).await }
    }
}

pub trait Async2RFn {
    type In1: Send;
    type In2: Send;
    type Out: Send;
    type E;

    #[allow(async_fn_in_trait)]
    fn invoke(
        &self,
        input1: Self::In1,
        input2: Self::In2,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send;

    /// Reifies `self` as an `async Fn`
    fn into_fn(
        self,
    ) -> impl Fn(
        Self::In1,
        Self::In2,
    ) -> Pin<Box<(dyn Future<Output = Result<Self::Out, Self::E>> + Send + 'static)>>
           + Send
           + Sync // optional, results from Self: Sync
           + 'static
           + Clone
    where
        Self: Send
            + Sync // optional if resulting Fn doesn't have to be Sync
            + Clone
            + 'static,
    {
        move |in1, in2| {
            let f = self.clone();
            Box::pin(async move {
                let output = f.invoke(in1, in2).await?;
                Ok(output)
            })
        }
    }
}

impl<F> Async2RFn for Arc<F>
where
    F: Async2RFn + Sync + Send,
{
    type In1 = F::In1;
    type In2 = F::In2;
    type Out = F::Out;
    type E = F::E;

    fn invoke(
        &self,
        in1: Self::In1,
        in2: Self::In2,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send {
        async { self.as_ref().invoke(in1, in2).await }
    }
}

pub trait Async3RFn {
    type In1: Send;
    type In2: Send;
    type In3: Send;
    type Out: Send;
    type E;

    #[allow(async_fn_in_trait)]
    fn invoke(
        &self,
        input1: Self::In1,
        input2: Self::In2,
        input3: Self::In3,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send;

    /// Reifies `self` as an `async Fn`
    fn into_fn(
        self,
    ) -> impl Fn(
        Self::In1,
        Self::In2,
        Self::In3,
    ) -> Pin<Box<(dyn Future<Output = Result<Self::Out, Self::E>> + Send + 'static)>>
           + Send
           + Sync // optional, results from Self: Sync
           + 'static
           + Clone
    where
        Self: Send
            + Sync // optional if resulting Fn doesn't have to be Sync
            + Clone
            + 'static,
    {
        move |in1, in2, in3| {
            let f = self.clone();
            Box::pin(async move {
                let output = f.invoke(in1, in2, in3).await?;
                Ok(output)
            })
        }
    }
}

impl<F> Async3RFn for Arc<F>
where
    F: Async3RFn + Sync + Send,
{
    type In1 = F::In1;
    type In2 = F::In2;
    type In3 = F::In3;
    type Out = F::Out;
    type E = F::E;

    fn invoke(
        &self,
        in1: Self::In1,
        in2: Self::In2,
        in3: Self::In3,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send {
        async { self.as_ref().invoke(in1, in2, in3).await }
    }
}

pub trait Async4RFn {
    type In1: Send;
    type In2: Send;
    type In3: Send;
    type In4: Send;
    type Out: Send;
    type E;

    #[allow(async_fn_in_trait)]
    fn invoke(
        &self,
        input1: Self::In1,
        input2: Self::In2,
        input3: Self::In3,
        input4: Self::In4,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send;

    /// Reifies `self` as an `async Fn`
    fn into_fn(
        self,
    ) -> impl Fn(
        Self::In1,
        Self::In2,
        Self::In3,
        Self::In4,
    ) -> Pin<Box<(dyn Future<Output = Result<Self::Out, Self::E>> + Send + 'static)>>
           + Send
           + Sync // optional, results from Self: Sync
           + 'static
           + Clone
    where
        Self: Send
            + Sync // optional if resulting Fn doesn't have to be Sync
            + Clone
            + 'static,
    {
        move |in1, in2, in3, in4| {
            let f = self.clone();
            Box::pin(async move {
                let output = f.invoke(in1, in2, in3, in4).await?;
                Ok(output)
            })
        }
    }
}

impl<F> Async4RFn for Arc<F>
where
    F: Async4RFn + Sync + Send,
{
    type In1 = F::In1;
    type In2 = F::In2;
    type In3 = F::In3;
    type In4 = F::In4;
    type Out = F::Out;
    type E = F::E;

    fn invoke(
        &self,
        in1: Self::In1,
        in2: Self::In2,
        in3: Self::In3,
        in4: Self::In4,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send {
        async { self.as_ref().invoke(in1, in2, in3, in4).await }
    }
}
