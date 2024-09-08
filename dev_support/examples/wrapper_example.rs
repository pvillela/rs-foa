use foa::{
    fun::AsyncRFn,
    wrapper::{Mappable, W},
};
use std::sync::Arc;

//=================
// Example extending a library wrapper with a trait

struct Discr1;
struct Discr2;

type Mappable1<T> = Mappable<Discr1, T>;
type Mappable2<T> = Mappable<Discr2, T>;

trait MappableExt<T> {
    fn map_str(&self, f: impl Fn(&T) -> String) -> impl MappableExt<String>;
}

impl<T> MappableExt<T> for Mappable1<T> {
    #[allow(refining_impl_trait)]
    fn map_str(&self, f: impl Fn(&T) -> String) -> Mappable1<String> {
        self.map(f)
    }
}

impl<T> MappableExt<T> for Mappable2<T> {
    #[allow(refining_impl_trait)]
    fn map_str(&self, f: impl Fn(&T) -> String) -> Mappable2<String> {
        let s = f(&self.0) + &f(&self.0);
        Mappable2::new(s)
    }
}

type M1 = Mappable1<Arc<i32>>;
type M2 = Mappable2<Arc<i32>>;

//=================
// Example leveraging a library wrapper with another layer of wrapping

struct MyW<T>(T);

struct AsyncRFnD;
impl<F> AsyncRFn for MyW<W<AsyncRFnD, F>>
where
    F: AsyncRFn + Sync,
{
    type In = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, input: F::In) -> Result<Self::Out, Self::E> {
        self.0.invoke(input).await
    }
}

struct AsyncRFnI;

impl AsyncRFn for AsyncRFnI {
    type In = i32;
    type Out = i32;
    type E = ();

    async fn invoke(&self, input: Self::In) -> Result<Self::Out, Self::E> {
        Ok(input * 2)
    }
}

#[tokio::main]
async fn main() {
    let m1 = M1::new(42.into());
    let s = m1.map_str(|x| x.to_string());
    println!("{s:?}");

    let m2 = M2::new(42.into());
    let s = m2.map_str(|x| x.to_string());
    println!("{s:?}");

    let f = MyW(W::<AsyncRFnD, _>::new(AsyncRFnI));
    let res = f.invoke(42).await;
    println!("{res:?}");
}
