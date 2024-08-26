use std::sync::Arc;

use foa::wrapper_discr::Mappable;

struct Discr1;
struct Discr2;

type Mappable1<T> = Mappable<T, Discr1>;
type Mappable2<T> = Mappable<T, Discr2>;

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

fn main() {
    let m1 = M1::new(42.into());
    let s = m1.map_str(|x| x.to_string());
    println!("{s:?}");

    let m2 = M2::new(42.into());
    let s = m2.map_str(|x| x.to_string());
    println!("{s:?}");
}
