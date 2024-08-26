use std::sync::Arc;

use foa::wrapper::Wrapper;

trait WrapperExt<T> {
    fn map_str(&self, f: impl FnMut(&T) -> String) -> Wrapper<String>;
}

impl<T> WrapperExt<T> for Wrapper<T> {
    fn map_str(&self, mut f: impl FnMut(&T) -> String) -> Wrapper<String> {
        Wrapper(f(self))
    }
}

type Foo = Wrapper<Arc<i32>>;

fn main() {
    let foo = Foo::new(42.into());
    let s = foo.map_str(|x| x.to_string());
    println!("{s:?}");
}
