use super::task_local::TaskLocal;
use crate::context::LocaleSelf;
use std::ops::Deref;

pub fn locale_from_task_local<T>(default: impl Deref<Target = str>) -> impl Deref<Target = str>
where
    T: TaskLocal,
    T::Value: LocaleSelf,
{
    T::with(|v| v.locale().unwrap_or_else(|| &default).to_owned())
}
