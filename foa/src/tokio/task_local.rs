use crate::{
    error::{BacktraceSpec, BasicKind, StdBoxError, INTERNAL_TAG},
    fun::{AsyncFn, AsyncFn2},
    Error,
};
use std::marker::PhantomData;
use tokio::task::LocalKey;

pub static TASK_LOCAL_ERROR: BasicKind<StdBoxError> =
    BasicKind::new("TASK_LOCAL_ERROR", None, &INTERNAL_TAG).with_backtrace(BacktraceSpec::Yes);

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
            .map_err(|err| TASK_LOCAL_ERROR.error_with_src(StdBoxError::new(err)))
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
        static MY_TL: TlWithLocale;
    }

    async fn foo_sfl<TL: TaskLocal>() -> (TL::Value, TL::Value)
    where
        TL::Value: Clone,
    {
        let v1 = TL::cloned_value();
        let v2 = TL::with(|v| v.clone());
        (v1, v2)
    }

    struct TlI;

    impl TaskLocal for TlI {
        type Value = TlWithLocale;

        fn local_key() -> &'static LocalKey<Self::Value> {
            &MY_TL
        }
    }

    struct FooI;

    impl AsyncFn for FooI {
        type In = ();
        type Out = Result<(TlWithLocale, TlWithLocale), ()>;

        async fn invoke(&self, _input: Self::In) -> Self::Out {
            Ok(foo_sfl::<TlI>().await)
        }
    }

    #[tokio::test]
    async fn test() {
        let h = tokio::spawn(async {
            let tlc = TlWithLocale {
                locale: "en-CA".into(),
            };
            invoke_tl_scoped::<_, TlI>(&FooI, tlc, ()).await
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
