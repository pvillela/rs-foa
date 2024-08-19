/// Used to convert a reference to another type with the same lifetie.
pub trait RefInto<'a, T> {
    fn ref_into(&'a self) -> T
    where
        T: 'a;
}

/// Used to construct the target type from a reference to the source type.
pub trait Make<T> {
    fn make(&self) -> T;
}

impl<T, F> Make<T> for F
where
    F: Fn() -> T,
{
    fn make(&self) -> T {
        self()
    }
}
