use crate::fun::{AsyncFn, AsyncFn2, AsyncRFn, AsyncRFn2};
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
    type Value: 'static;

    fn local_key() -> &'static LocalKey<Self::Value>;

    fn with<U>(f: impl FnOnce(&Self::Value) -> U) -> U {
        let lk = Self::local_key();
        lk.with(|v| f(v))
    }

    fn cloned_value() -> Self::Value
    where
        Self::Value: Clone,
    {
        Self::with(|v| v.clone())
    }
}

#[derive(Clone)]
struct TlScoped<F, TL>(F, PhantomData<TL>);

impl<F, TL> AsyncRFn2 for TlScoped<F, TL>
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
        let output = lk.scope(value, self.0.invoke(input)).await?;
        Ok(output)
    }
}

impl<F, TL> AsyncFn2 for TlScoped<F, TL>
where
    TL: TaskLocal + Sync,
    TL::Value: Send,
    F: AsyncFn + Sync,
{
    type In1 = TL::Value;
    type In2 = F::In;
    type Out = F::Out;

    async fn invoke(&self, value: Self::In1, input: Self::In2) -> Self::Out {
        let lk = TL::local_key();
        lk.scope(value, self.0.invoke(input)).await
    }
}

pub fn tl_scoped_old<'a, F, TL>(
    f: F,
) -> impl AsyncRFn2<In1 = TL::Value, In2 = F::In, Out = F::Out, E = F::E> + 'a
where
    TL: TaskLocal + Sync + 'static,
    TL::Value: Send,
    F: AsyncRFn + Sync + 'a,
{
    TlScoped(f, PhantomData::<TL>)
}

pub fn tl_scoped<'a, F, TL>(f: F) -> impl AsyncFn2<In1 = TL::Value, In2 = F::In, Out = F::Out> + 'a
where
    TL: TaskLocal + Sync + 'static,
    TL::Value: Send,
    F: AsyncFn + Sync + 'a,
{
    TlScoped(f, PhantomData::<TL>)
}

pub async fn invoke_tl_scoped_old<F, TL>(f: &F, in1: TL::Value, in2: F::In) -> Result<F::Out, F::E>
where
    TL: TaskLocal + Sync,
    TL::Value: Send,
    F: AsyncRFn + Sync,
{
    TlScoped(f, PhantomData::<TL>).invoke(in1, in2).await
}

pub async fn invoke_tl_scoped<F, TL>(f: &F, in1: TL::Value, in2: F::In) -> F::Out
where
    TL: TaskLocal + Sync,
    TL::Value: Send,
    F: AsyncFn + Sync,
{
    TlScoped(f, PhantomData::<TL>).invoke(in1, in2).await
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
        <CTX::TaskLocal as TaskLocal>::Value,
        <CTX::TaskLocal as TaskLocal>::Value,
    )
    where
        <CTX::TaskLocal as TaskLocal>::Value: Clone,
    {
        let v1 = CTX::TaskLocal::cloned_value();
        let v2 = CTX::TaskLocal::with(|v| v.clone());
        (v1, v2)
    }

    struct Ctx<const K: u8 = 0>;

    impl TaskLocal for Ctx<1> {
        type Value = TlWithLocale;

        fn local_key() -> &'static LocalKey<Self::Value> {
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
            invoke_tl_scoped_old::<_, <Ctx as TaskLocalCtx>::TaskLocal>(&FooI(Ctx), tlc, ()).await
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
