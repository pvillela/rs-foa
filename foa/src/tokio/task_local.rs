use crate::fun::AsyncRFn;
use std::marker::PhantomData;
use tokio::task::LocalKey;

pub trait TaskLocalCtx<D = ()> {
    type TaskLocal: TaskLocal<D>;
}

pub trait TaskLocal<D = ()> {
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

struct TlScoped<CTX, F, D = ()>(F, PhantomData<(CTX, D)>);

impl<CTX, F, D> AsyncRFn for TlScoped<CTX, F, D>
where
    CTX: TaskLocalCtx<D> + Sync,
    <CTX::TaskLocal as TaskLocal<D>>::ValueType: Send,
    F: AsyncRFn + Sync,
    D: Sync,
{
    type In = (<CTX::TaskLocal as TaskLocal<D>>::ValueType, F::In);
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, (value, input): Self::In) -> Result<Self::Out, Self::E> {
        let lk = CTX::TaskLocal::local_key();
        let output = lk.scope(value, self.0.invoke(input)).await?;
        Ok(output)
    }
}

pub async fn tl_scoped<CTX, F, D>(
    f: F,
) -> impl AsyncRFn<In = (<CTX::TaskLocal as TaskLocal<D>>::ValueType, F::In), Out = F::Out, E = F::E>
where
    CTX: TaskLocalCtx<D> + Sync,
    <CTX::TaskLocal as TaskLocal<D>>::ValueType: Send,
    F: AsyncRFn + Sync,
    D: Sync,
{
    TlScoped(f, PhantomData::<(CTX, D)>)
}

pub async fn invoke_tl_scoped<CTX, F, D>(
    f: F,
    input: (<CTX::TaskLocal as TaskLocal<D>>::ValueType, F::In),
) -> Result<F::Out, F::E>
where
    CTX: TaskLocalCtx<D> + Sync,
    <CTX::TaskLocal as TaskLocal<D>>::ValueType: Send,
    F: AsyncRFn + Sync,
    D: Sync,
{
    TlScoped(f, PhantomData::<(CTX, D)>).invoke(input).await
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
            invoke_tl_scoped::<Ctx, _, _>(FooI(Ctx), (tlc, ())).await
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
