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

pub trait TaskLocalScopedFn<CTX: TaskLocalCtx<D>, D = ()> {
    type In;
    type Out;

    #[allow(async_fn_in_trait)]
    async fn call(input: Self::In) -> Self::Out;

    #[allow(async_fn_in_trait)]
    async fn tl_scoped(
        value: <CTX::TaskLocal as TaskLocal<D>>::ValueType,
        input: Self::In,
    ) -> Self::Out {
        let lk = CTX::TaskLocal::local_key();
        lk.scope(value, Self::call(input)).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::marker::PhantomData;

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

    struct FooI<CTX>(PhantomData<CTX>);

    impl TaskLocalScopedFn<Ctx> for FooI<Ctx> {
        type In = ();
        type Out = (TlWithLocale, TlWithLocale);

        async fn call(_input: Self::In) -> Self::Out {
            foo_sfl::<Ctx>().await
        }
    }

    #[tokio::test]
    async fn test() {
        let h = tokio::spawn(async {
            let tlc = TlWithLocale {
                locale: "en-CA".into(),
            };
            FooI::<Ctx>::tl_scoped(tlc, ()).await
        });
        let foo_out = h.await.unwrap();
        assert_eq!(
            foo_out,
            (
                TlWithLocale {
                    locale: "en-CA".into(),
                },
                TlWithLocale {
                    locale: "en-CA".into(),
                }
            )
        );
    }
}
