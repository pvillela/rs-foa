use tokio::task::LocalKey;

pub trait TaskLocalCtx<D = ()> {
    type ValueType: 'static;

    fn local_key() -> &'static LocalKey<Self::ValueType>;

    fn with_tl<U>(f: impl FnOnce(&Self::ValueType) -> U) -> U {
        let lk = Self::local_key();
        lk.with(|v| f(v))
    }

    fn cloned_tl_value() -> Self::ValueType
    where
        Self::ValueType: Clone,
    {
        Self::with_tl(|v| v.clone())
    }
}

pub trait TaskLocalScopedFn<CTX: TaskLocalCtx<D>, D = ()> {
    type In;
    type Out;

    #[allow(async_fn_in_trait)]
    async fn call(input: Self::In) -> Self::Out;

    #[allow(async_fn_in_trait)]
    async fn tl_scoped(value: CTX::ValueType, input: Self::In) -> Self::Out {
        let lk = CTX::local_key();
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

    async fn foo_sfl<CTX: TaskLocalCtx<ValueType: Clone>>() -> (CTX::ValueType, CTX::ValueType) {
        let v1 = CTX::cloned_tl_value();
        let v2 = CTX::with_tl(|v| v.clone());
        (v1, v2)
    }

    struct Ctx;

    impl TaskLocalCtx for Ctx {
        type ValueType = TlWithLocale;

        fn local_key() -> &'static LocalKey<Self::ValueType> {
            &CTX_TL
        }
    }

    struct FooI<CTX>(PhantomData<CTX>);

    impl<CTX: TaskLocalCtx<ValueType: Clone>> TaskLocalScopedFn<CTX> for FooI<CTX> {
        type In = ();
        type Out = (CTX::ValueType, CTX::ValueType);

        async fn call(_input: Self::In) -> Self::Out {
            foo_sfl::<CTX>().await
        }
    }

    #[tokio::test]
    async fn test() {
        let h = tokio::spawn(async {
            let tlc = TlWithLocale {
                locale: "en-ca".into(),
            };
            FooI::<Ctx>::tl_scoped(tlc, ()).await
        });
        let foo_out = h.await.unwrap();
        assert_eq!(
            foo_out,
            (
                TlWithLocale {
                    locale: "en-ca".into(),
                },
                TlWithLocale {
                    locale: "en-ca".into(),
                }
            )
        );
    }
}
