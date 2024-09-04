use super::task_local::TaskLocal;
use crate::context::LocaleSelf;
use std::ops::Deref;

pub fn locale_from_task_local<T, D>() -> impl Deref<Target = str>
where
    T: TaskLocal<D>,
    T::ValueType: LocaleSelf,
{
    T::with(|v| v.locale().to_owned())
}
