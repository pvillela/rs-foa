//! Explorations on a more general version of [`super::newper`] that has an additional
//! discriminant type parameter.

use std::{
    borrow::Borrow,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};

//=================
// Wrapper

/// Generic wrapper to enable the addition of new methods to the wrapped type,
/// with a discriminant type parameter `P` to enable different wrappings of the same target
/// and implementing methods on wrapped type in other crates.
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct W<D, T, Ph = ()>(pub T, pub PhantomData<(D, Ph)>);

impl<D, T, Ph> W<D, T, Ph> {
    pub fn new(value: T) -> W<D, T, Ph> {
        Self(value, PhantomData)
    }

    pub fn value(&self) -> &T {
        &self.0
    }
}

impl<D, T, Ph> Clone for W<D, T, Ph>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        W(self.0.clone(), PhantomData)
    }
}

impl<D, T, Ph> Debug for W<D, T, Ph>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&self.0 as &dyn Debug).fmt(f)
    }
}

impl<D, T, Ph> From<T> for W<D, T, Ph> {
    fn from(value: T) -> Self {
        Self(value, PhantomData)
    }
}

impl<D, T, Ph> Deref for W<D, T, Ph> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D, T, Ph> DerefMut for W<D, T, Ph> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<D, T, Ph> AsRef<T> for W<D, T, Ph> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<D, T, Ph> Borrow<T> for W<D, T, Ph> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<D, T, Ph> Borrow<T> for W<D, Box<T>, Ph> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<D, T, Ph> Borrow<T> for W<D, Arc<T>, Ph> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<D, T, Ph> Borrow<T> for W<D, Rc<T>, Ph> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<D, T, Ph> IntoIterator for W<D, T, Ph>
where
    T: IntoIterator,
{
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, D, T, Ph> IntoIterator for &'a W<D, T, Ph>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, D, T, Ph> IntoIterator for &'a mut W<D, T, Ph>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

//=================
// Mappable

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MappableDiscr<D>(D);

/// Specializetion of [Wrapper] that adds a [`map`](Self::map) method.
pub type Mappable<D, T, Ph = ()> = W<MappableDiscr<D>, T, Ph>;

impl<D, T, Ph> Mappable<D, T, Ph> {
    /// Transforms `self` into a target [`Mappable<U, P>`] whose wrapped value is the result of applying `f` to
    /// `self`'s wrapped value.
    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> Mappable<D, U, Ph> {
        Mappable::new(f(&self.0))
    }
}
