//===========================
// region:      --- Payload

use std::any::Any;
use std::fmt::Debug;
use std::mem::replace;

pub trait Payload: Debug + Send + Sync + 'static {}

impl<T> Payload for T where T: Debug + Send + Sync + 'static {}

// endregion:   --- Payload

//===========================
// region:      --- MaybePayload

/// Similar to [`Option`], named after Haskell equivalent.
pub enum MaybePayload<T: Payload> {
    Nothing,
    Just(T),
}

impl<T: Payload> MaybePayload<T> {
    fn just(self) -> Option<T> {
        match self {
            Self::Nothing => None,
            Self::Just(e) => Some(e),
        }
    }
}

impl<T: Payload> Debug for MaybePayload<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nothing => f.write_str(""),
            Self::Just(e) => Debug::fmt(e, f),
        }
    }
}

// endregion:   --- MaybePayload

//===========================
// region:      --- BoxPayload

trait PayloadPriv: Payload {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> PayloadPriv for T
where
    T: Payload,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct BoxPayload(Box<dyn PayloadPriv>);

impl BoxPayload {
    pub fn new(inner: impl Payload) -> Self {
        Self(Box::new(MaybePayload::Just(inner)))
    }

    pub fn downcast_ref<T: Payload>(&self) -> Option<&T> {
        let err_box_dyn = &self.0;
        err_box_dyn
            .as_any()
            .downcast_ref::<MaybePayload<T>>()
            .map(|w| match w {
                MaybePayload::Nothing => unreachable!("invalid state"),
                MaybePayload::Just(e) => e,
            })
    }

    /// Not a very useful method
    pub fn downcast_mut<T: Payload>(&mut self) -> Option<&mut T> {
        let err_box_dyn = &mut self.0;
        err_box_dyn
            .as_any_mut()
            .downcast_mut::<MaybePayload<T>>()
            .map(|w| match w {
                MaybePayload::Nothing => unreachable!("invalid state"),
                MaybePayload::Just(e) => e,
            })
    }

    pub fn downcast<T: Payload>(mut self) -> Result<T, Self> {
        let err_box_dyn = &mut self.0;
        let err_dyn_any = err_box_dyn.as_any_mut();
        if err_dyn_any.is::<MaybePayload<T>>() {
            let err_with_null_r = err_dyn_any
                .downcast_mut::<MaybePayload<T>>()
                .expect("downcast success previously confirmed");
            let err_with_null_v = replace(err_with_null_r, MaybePayload::Nothing);
            Ok(err_with_null_v.just().unwrap())
        } else {
            Err(self)
        }
    }

    /// If the boxed value is of type `T`, returns `Err(f(value))`; otherwise, returns `Ok(self)`.
    /// This unusual signature facilitates chaining of calls of this method with different types.
    pub fn with_downcast<T: Payload, U>(self, f: impl FnOnce(T) -> U) -> Result<Self, U> {
        let res = self.downcast::<T>();
        match res {
            Ok(t) => Err(f(t)),
            Err(err) => Ok(err),
        }
    }
}

impl Debug for BoxPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

// endregion:   --- BoxPayload
