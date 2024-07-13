//! The trait defined here was adapted from https://github.com/rust-lang/rust/issues/113495#issuecomment-1627640952
//! in response to my issue https://github.com/rust-lang/rust/issues/113495;
//! Enhanced by https://github.com/rust-lang/rust/issues/113495#issuecomment-1728150795.

use std::{future::Future, pin::Pin};

/// Represents an async function with single argument that is a reference.
pub trait AsyncBorrowFn1b1<'a, A: ?Sized + 'a, Out>: Fn(&'a A) -> Self::Fut + Send + Sync {
    type Fut: Future<Output = Out> + Send + Sync + 'a;
}

impl<'a, A, Out, F, Fut> AsyncBorrowFn1b1<'a, A, Out> for F
where
    A: ?Sized + 'a,
    F: Fn(&'a A) -> Fut + Send + Sync + 'a,
    Fut: Future<Output = Out> + Send + Sync + 'a,
{
    type Fut = Fut;
}

/// Represents an async function with 2 arguments; the first is not a reference, the last is a reference.
pub trait AsyncBorrowFn2b2<'a, A1, A2: ?Sized + 'a, Out>:
    Fn(A1, &'a A2) -> Self::Fut + Send + Sync
{
    type Fut: Future<Output = Out> + Send + Sync + 'a;
}

impl<'a, A1, A2, Out, F, Fut> AsyncBorrowFn2b2<'a, A1, A2, Out> for F
where
    A2: ?Sized + 'a,
    F: Fn(A1, &'a A2) -> Fut + Send + Sync + 'a,
    Fut: Future<Output = Out> + Send + Sync + 'a,
{
    type Fut = Fut;
}

/// Represents an async function with 3 arguments; the first 2 are not references, the last is a reference.
pub trait AsyncBorrowFn3b3<'a, A1, A2, A3: ?Sized + 'a, Out>:
    Fn(A1, A2, &'a A3) -> Self::Fut + Send + Sync
{
    type Fut: Future<Output = Out> + Send + Sync + 'a;
}

impl<'a, A1, A2, A3, Out, F, Fut> AsyncBorrowFn3b3<'a, A1, A2, A3, Out> for F
where
    A3: ?Sized + 'a,
    F: Fn(A1, A2, &'a A3) -> Fut + Send + Sync + 'a,
    Fut: Future<Output = Out> + Send + Sync + 'a,
{
    type Fut = Fut;
}

/// Represents an async function with 4 arguments; the first 3 are not references, the last is a reference.
pub trait AsyncBorrowFn4b4<'a, A1, A2, A3, A4: ?Sized + 'a, Out>:
    Fn(A1, A2, A3, &'a A4) -> Self::Fut + Send + Sync
{
    type Fut: Future<Output = Out> + Send + Sync + 'a;
}

impl<'a, A1, A2, A3, A4, Out, F, Fut> AsyncBorrowFn4b4<'a, A1, A2, A3, A4, Out> for F
where
    A4: ?Sized + 'a,
    F: Fn(A1, A2, A3, &'a A4) -> Fut + Send + Sync + 'a,
    Fut: Future<Output = Out> + Send + Sync + 'a,
{
    type Fut = Fut;
}

/// Partial application for async function, where the resulting closure returns a box-pinned future.
pub fn partial_apply_async_borrow_fn_2b2_boxpin<A1, A2, T>(
    f: impl for<'a> AsyncBorrowFn2b2<'a, A1, A2, T>,
    a1: A1,
) -> impl for<'a> Fn(&'a A2) -> Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>> + Send + Sync
where
    A1: Clone + Send + Sync,
    A2: ?Sized, // optional Sized relaxation
{
    move |a2| {
        let y = f(a1.clone(), a2);
        Box::pin(y)
    }
}

///Partial application for async function, where the result is an AsyncBorrowFn1a1.
///
/// The code below doesn't compile, thus the need for `nudge_inference`
/// (see https://github.com/rust-lang/rust/issues/113495#issuecomment-1728150795)
/// in this function.
/// ```
/// pub fn partial_apply<A1, A2, T>(
///     f: impl for<'a> AsyncBorrowFn2b2<'a, A1, &'a A2, T> + 'static,
///     a1: A1,
/// ) -> impl for<'a> AsyncBorrowFn1b1<'a, &'a A2, T>
/// where
///     A1: Clone + Send + Sync + 'static,
///     A2: ?Sized + 'static,
/// {
///     move |a2| f(a1.clone(), a2)
/// }
/// ```
pub fn partial_apply_async_borrow_fn_2b2<A1, A2, T, F>(
    f: F,
    a1: A1,
) -> impl for<'a> AsyncBorrowFn1b1<'a, A2, T>
where
    A1: Clone + Send + Sync + 'static,
    A2: ?Sized + 'static,
    F: for<'a> AsyncBorrowFn2b2<'a, A1, A2, T> + 'static,
{
    fn nudge_inference<A1, A2, T, F, C>(closure: C) -> C
    where
        A1: Clone + 'static,
        A2: ?Sized + 'static,
        F: for<'a> AsyncBorrowFn2b2<'a, A1, A2, T> + 'static,

        // this promotes the literal `|a2| …` closure to "infer"
        // (get imbued with) the right higher-order fn signature.
        // See https://docs.rs/higher-order-closure for more info
        C: Fn(&A2) -> <F as AsyncBorrowFn2b2<'_, A1, A2, T>>::Fut,
    {
        closure
    }

    nudge_inference::<A1, A2, T, F, _>(move |a2| f(a1.clone(), a2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    trait Trt {
        fn value(&self) -> u32;
    }

    impl Trt for u32 {
        fn value(&self) -> u32 {
            *self
        }
    }

    async fn f0(i: u32, j: &u32) -> u32 {
        tokio::time::sleep(Duration::from_millis(10)).await;
        i + j
    }

    async fn f1<'a>(i: u32, j: &(dyn Trt + Send + Sync + 'a)) -> u32 {
        i + j.value() + 1
    }

    /// With a type alias, there is no need to add a lifetime parameter, unlike
    /// what had to be done for `f1` above, for the partial_apply functions to accept `f2`. Why?
    type DynTrt = dyn Trt + Send + Sync;

    async fn f2(i: u32, j: &DynTrt) -> u32 {
        i + j.value() + 2
    }

    #[tokio::test]
    async fn test_all() {
        let f_part = partial_apply_async_borrow_fn_2b2_boxpin(f0, 40);
        assert_eq!(42, f_part(&2).await);

        let f_part = partial_apply_async_borrow_fn_2b2(f0, 40);
        assert_eq!(42, f_part(&2).await);

        // The commented-out lines below don't compile
        // let g = |x, y: &u32| f0(x, y);

        let f_part = partial_apply_async_borrow_fn_2b2_boxpin(f1, 40);
        assert_eq!(43, f_part(&2).await);

        let f_part = partial_apply_async_borrow_fn_2b2(f1, 40);
        assert_eq!(43, f_part(&2).await);

        let f_part = partial_apply_async_borrow_fn_2b2_boxpin(f2, 40);
        assert_eq!(44, f_part(&2).await);

        let f_part = partial_apply_async_borrow_fn_2b2(f2, 40);
        assert_eq!(44, f_part(&2).await);
    }
}
