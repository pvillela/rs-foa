use crate::{
    error::{BacktraceSpec, BasicKind, INTERNAL_TAG},
    fun::{AsyncFn, AsyncFn2},
    Error,
};
use std::marker::PhantomData;
use tokio::task::LocalKey;

pub static TASK_LOCAL_ERROR: BasicKind<true> = BasicKind::new(
    "TASK_LOCAL_ERROR",
    None,
    BacktraceSpec::Yes,
    Some(&INTERNAL_TAG),
);

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

    fn try_with<U>(f: impl FnOnce(&Self::Value) -> U) -> Result<U, Error> {
        let lk = Self::local_key();
        lk.try_with(|v| f(v))
            .map_err(|err| TASK_LOCAL_ERROR.error(err))
    }

    fn with<U>(f: impl FnOnce(&Self::Value) -> U) -> U {
        Self::try_with(f).expect("task-local value not set")
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

pub fn tl_scoped<'a, F, TL>(f: F) -> impl AsyncFn2<In1 = TL::Value, In2 = F::In, Out = F::Out> + 'a
where
    TL: TaskLocal + Sync + 'a,
    TL::Value: Send,
    F: AsyncFn + Sync + 'a,
{
    TlScoped(f, PhantomData::<TL>)
}

pub async fn invoke_tl_scoped<F, TL>(f: &F, in1: TL::Value, in2: F::In) -> F::Out
where
    TL: TaskLocal + Sync,
    TL::Value: Send,
    F: AsyncFn + Sync,
{
    tl_scoped::<_, TL>(f).invoke(in1, in2).await
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

    impl AsyncFn for FooI<Ctx> {
        type In = ();
        type Out = Result<(TlWithLocale, TlWithLocale), ()>;

        async fn invoke(&self, _input: Self::In) -> Self::Out {
            Ok(foo_sfl::<Ctx>().await)
        }
    }

    #[tokio::test]
    async fn test() {
        let h = tokio::spawn(async {
            let tlc = TlWithLocale {
                locale: "en-CA".into(),
            };
            invoke_tl_scoped::<_, <Ctx as TaskLocalCtx>::TaskLocal>(&FooI(Ctx), tlc, ()).await
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
