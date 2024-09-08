use crate::{
    fun::{Async2RFn, Async3RFn, Async4RFn, AsyncRFn},
    wrapper::W,
};
use std::marker::PhantomData;
use tokio::task::LocalKey;

pub trait TaskLocalCtx {
    type TaskLocal: TaskLocal;
}

/// Represents state stored in a task-local variable.
///
/// Without loss of generality, if a type `T` needs to implement [`TaskLocal`] for [`ValueType`](TaskLocal::ValueType)s
/// `S1` and `S2`, then `T` can implement `TaskLocal` with `type ValueType = (S1, S2)`.
pub trait TaskLocal {
    type ValueType: 'static;

    fn local_key() -> &'static LocalKey<Self::ValueType>;

    fn with<U>(f: impl FnOnce(&Self::ValueType) -> U) -> U {
        let lk = Self::local_key();
        lk.with(|v| f(v))
    }

    fn cloned_value() -> Self::ValueType
    where
        Self::ValueType: Clone,
    {
        Self::with(|v| v.clone())
    }
}

#[derive(Clone)]
struct TlScoped<'a, CTX, F>(&'a F, PhantomData<CTX>);

impl<'a, CTX, F> AsyncRFn for TlScoped<'a, CTX, F>
where
    CTX: TaskLocalCtx + Sync,
    <CTX::TaskLocal as TaskLocal>::ValueType: Send,
    F: AsyncRFn + Sync,
{
    type In = (<CTX::TaskLocal as TaskLocal>::ValueType, F::In);
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, (value, input): Self::In) -> Result<Self::Out, Self::E> {
        let lk = CTX::TaskLocal::local_key();
        let output = lk.scope(value, self.0.invoke(input)).await?;
        Ok(output)
    }
}

pub fn tl_scoped<'a, CTX, F>(
    f: &'a F,
) -> impl AsyncRFn<In = (<CTX::TaskLocal as TaskLocal>::ValueType, F::In), Out = F::Out, E = F::E> + 'a
where
    CTX: TaskLocalCtx + Sync + 'static,
    <CTX::TaskLocal as TaskLocal>::ValueType: Send,
    F: AsyncRFn + Sync,
{
    TlScoped(f, PhantomData::<CTX>)
}

pub async fn invoke_tl_scoped<CTX, F>(
    f: &F,
    input: (<CTX::TaskLocal as TaskLocal>::ValueType, F::In),
) -> Result<F::Out, F::E>
where
    CTX: TaskLocalCtx + Sync,
    <CTX::TaskLocal as TaskLocal>::ValueType: Send,
    F: AsyncRFn + Sync,
{
    TlScoped(f, PhantomData::<CTX>).invoke(input).await
}

/// Discriminant for conversion of [`AsyncRFn`] to [`Async2RFn`] in task-local context using [`W`].
pub struct Async2RFnTlD;
impl<CTX, F, TLV> Async2RFn for W<F, Async2RFnTlD, CTX>
where
    CTX: TaskLocalCtx<TaskLocal: TaskLocal<ValueType = TLV>> + Sync + Send + 'static,
    TLV: Send,
    F: AsyncRFn + Sync + Send,
{
    type In1 = TLV;
    type In2 = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, in1: Self::In1, in2: Self::In2) -> Result<Self::Out, Self::E> {
        invoke_tl_scoped::<CTX, _>(&self.0, (in1, in2)).await
    }
}

/// Discriminant for conversion of [`AsyncRFn`] to [`Async3RFn`] in task-local context using [`W`].
pub struct Async3RFnTlD;
impl<CTX, F, TLV1, TLV2> Async3RFn for W<F, Async3RFnTlD, CTX>
where
    CTX: TaskLocalCtx<TaskLocal: TaskLocal<ValueType = (TLV1, TLV2)>> + Sync + Send + 'static,
    TLV1: Send,
    TLV2: Send,
    F: AsyncRFn + Sync + Send,
{
    type In1 = TLV1;
    type In2 = TLV2;
    type In3 = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(
        &self,
        in1: Self::In1,
        in2: Self::In2,
        in3: Self::In3,
    ) -> Result<Self::Out, Self::E> {
        invoke_tl_scoped::<CTX, _>(&self.0, ((in1, in2), in3)).await
    }
}

/// Discriminant for conversion of [`AsyncRFn`] to [`Async4RFn`] in task-local context using [`W`].
pub struct Async4RFnTlD;
impl<CTX, F, TLV1, TLV2, TLV3> Async4RFn for W<F, Async4RFnTlD, CTX>
where
    CTX: TaskLocalCtx<TaskLocal: TaskLocal<ValueType = (TLV1, TLV2, TLV3)>> + Sync + Send + 'static,
    TLV1: Send,
    TLV2: Send,
    TLV3: Send,
    F: AsyncRFn + Sync + Send,
{
    type In1 = TLV1;
    type In2 = TLV2;
    type In3 = TLV3;
    type In4 = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(
        &self,
        in1: Self::In1,
        in2: Self::In2,
        in3: Self::In3,
        in4: Self::In4,
    ) -> Result<Self::Out, Self::E> {
        invoke_tl_scoped::<CTX, _>(&self.0, ((in1, in2, in3), in4)).await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[allow(unused)]
    #[derive(Debug, Clone, PartialEq)]
    struct TlWithLocale {
        locale: String,
    }

    tokio::task_local! {
        static CTX_TL: TlWithLocale;
    }

    async fn foo_sfl<CTX: TaskLocalCtx>() -> (
        <CTX::TaskLocal as TaskLocal>::ValueType,
        <CTX::TaskLocal as TaskLocal>::ValueType,
    )
    where
        <CTX::TaskLocal as TaskLocal>::ValueType: Clone,
    {
        let v1 = CTX::TaskLocal::cloned_value();
        let v2 = CTX::TaskLocal::with(|v| v.clone());
        (v1, v2)
    }

    struct Ctx<const K: u8 = 0>;

    impl TaskLocal for Ctx<1> {
        type ValueType = TlWithLocale;

        fn local_key() -> &'static LocalKey<Self::ValueType> {
            &CTX_TL
        }
    }

    impl TaskLocalCtx for Ctx {
        type TaskLocal = Ctx<1>;
    }

    struct FooI<CTX>(CTX);

    impl AsyncRFn for FooI<Ctx> {
        type In = ();
        type Out = (TlWithLocale, TlWithLocale);
        type E = ();

        async fn invoke(&self, _input: Self::In) -> Result<Self::Out, ()> {
            Ok(foo_sfl::<Ctx>().await)
        }
    }

    #[tokio::test]
    async fn test() {
        let h = tokio::spawn(async {
            let tlc = TlWithLocale {
                locale: "en-CA".into(),
            };
            invoke_tl_scoped::<Ctx, _>(&FooI(Ctx), (tlc, ())).await
        });
        let foo_out = h.await.unwrap();
        assert_eq!(
            foo_out,
            Ok((
                TlWithLocale {
                    locale: "en-CA".into(),
                },
                TlWithLocale {
                    locale: "en-CA".into(),
                }
            ))
        );
    }
}
