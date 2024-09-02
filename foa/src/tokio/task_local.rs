use tokio::task::LocalKey;

pub trait TaskLocalCtx<D = ()> {
    type TaskLocalType: Clone + 'static;

    fn get_static() -> &'static LocalKey<Self::TaskLocalType>;

    fn tl_value() -> Self::TaskLocalType {
        let lk = Self::get_static();
        lk.with(|tlc| tlc.clone())
    }
}

pub trait TaskLocalScopedFn<CTX: TaskLocalCtx<D>, D = ()> {
    type In;
    type Out;

    #[allow(async_fn_in_trait)]
    async fn call(input: Self::In) -> Self::Out;

    #[allow(async_fn_in_trait)]
    async fn tl_scoped(value: CTX::TaskLocalType, input: Self::In) -> Self::Out {
        let lk = CTX::get_static();
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

    async fn foo_sfl<CTX: TaskLocalCtx>() -> CTX::TaskLocalType {
        CTX::tl_value()
    }

    struct Ctx;

    impl TaskLocalCtx for Ctx {
        type TaskLocalType = TlWithLocale;

        fn get_static() -> &'static LocalKey<Self::TaskLocalType> {
            &CTX_TL
        }
    }

    struct FooI<CTX>(PhantomData<CTX>);

    impl<CTX: TaskLocalCtx> TaskLocalScopedFn<CTX> for FooI<CTX> {
        type In = ();
        type Out = CTX::TaskLocalType;

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
            TlWithLocale {
                locale: "en-ca".into(),
            }
        );
    }
}
