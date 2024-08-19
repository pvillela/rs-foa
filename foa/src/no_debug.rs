//! Generic wrapper to disable Debug for the wrapped type.
//! Based on [`crate::Wrapper`] and https://github.com/rust-lang/rust/issues/37009#issuecomment-2209496680.

use std::{
    borrow::Borrow,
    fmt::Debug,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};

/// Generic wrapper to disable Debug for the wrapped type.
#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct NoDebug<T>(pub T);

impl<T> NoDebug<T> {
    pub fn new(value: T) -> NoDebug<T> {
        Self(value)
    }

    pub fn wrap(value: T) -> NoDebug<T> {
        Self(value)
    }

    pub fn value(&self) -> &T {
        &self.0
    }
}

impl<T> Debug for NoDebug<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("skipped").finish()
    }
}

impl<T> From<T> for NoDebug<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for NoDebug<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NoDebug<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> AsRef<T> for NoDebug<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Borrow<T> for NoDebug<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> Borrow<T> for NoDebug<Box<T>> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T> Borrow<T> for NoDebug<Arc<T>> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T> Borrow<T> for NoDebug<Rc<T>> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T> IntoIterator for NoDebug<T>
where
    T: IntoIterator,
{
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a NoDebug<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut NoDebug<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
