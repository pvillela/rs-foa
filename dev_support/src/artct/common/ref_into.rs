/// Used to convert a reference to another type with the same lifecycle.
pub trait RefInto<'a, T> {
    fn ref_into(&'a self) -> T
    where
        T: 'a;
}
