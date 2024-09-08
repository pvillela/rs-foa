//! Generic wrapper to facilitate the addition of new methods to the wrapped type.

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

/// Generic wrapper to enable the addition of new methods to a wrapped type `T`,
///
/// Type parameters:
/// - `T` is the wrapped target type.
/// - `D` is a discriminant type that enables different wrappings of the same target type.
/// - `Ph` supports the inclusion of a phantomed type to the signature of `W`.
///   `Ph` can be a tuple of types if multiple phantom types are required.
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct W<T, D = (), Ph = ()>(pub T, pub PhantomData<(D, Ph)>);

impl<T, D, Ph> W<T, D, Ph> {
    pub fn new(value: T) -> W<T, D, Ph> {
        Self(value, PhantomData)
    }

    pub fn value(&self) -> &T {
        &self.0
    }
}

impl<T, D, Ph> Clone for W<T, D, Ph>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        W(self.0.clone(), PhantomData)
    }
}

impl<T, D, Ph> Debug for W<T, D, Ph>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&self.0 as &dyn Debug).fmt(f)
    }
}

impl<T, D, Ph> From<T> for W<T, D, Ph> {
    fn from(value: T) -> Self {
        Self(value, PhantomData)
    }
}

impl<T, D, Ph> Deref for W<T, D, Ph> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, D, Ph> DerefMut for W<T, D, Ph> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T, D, Ph> AsRef<T> for W<T, D, Ph> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T, D, Ph> Borrow<T> for W<T, D, Ph> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T, D, Ph> Borrow<T> for W<Box<T>, D, Ph> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T, D, Ph> Borrow<T> for W<Arc<T>, D, Ph> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T, D, Ph> Borrow<T> for W<Rc<T>, D, Ph> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T, D, Ph> IntoIterator for W<T, D, Ph>
where
    T: IntoIterator,
{
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, D, Ph> IntoIterator for &'a W<T, D, Ph>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, D, Ph> IntoIterator for &'a mut W<T, D, Ph>
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
// Example of complex usage of `W`.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MappableDiscr<D>(D);

/// Specialization of [`W`].
pub type Mappable<T, D, Ph = ()> = W<T, MappableDiscr<D>, Ph>;

/// Specialization of [`W`] that adds a [`map`](Self::map) method.
impl<T, D, Ph> Mappable<T, D, Ph> {
    /// Transforms `self` into a target [`Mappable<U, P>`] whose wrapped value is the result of applying `f` to
    /// `self`'s wrapped value.
    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> Mappable<U, D, Ph> {
        Mappable::new(f(&self.0))
    }
}
