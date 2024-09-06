pub trait Itself {
    fn it() -> Self;
}

pub trait Make<T> {
    fn make() -> T;
}
