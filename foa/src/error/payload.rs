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

pub(crate) trait PayloadPriv: Payload {
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

pub struct BoxPayload(pub(crate) Box<dyn PayloadPriv>);

impl BoxPayload {
    pub fn new(inner: impl Payload) -> Self {
        Self(Box::new(MaybePayload::Just(inner)))
    }

    pub fn is<T: Payload>(&self) -> bool {
        self.0.as_ref().as_any().is::<MaybePayload<T>>()
    }

    pub fn downcast_ref<T: Payload>(&self) -> Option<&T> {
        let pld_box_dyn = &self.0;
        pld_box_dyn
            .as_any()
            .downcast_ref::<MaybePayload<T>>()
            .map(|w| match w {
                MaybePayload::Nothing => unreachable!("invalid state"),
                MaybePayload::Just(e) => e,
            })
    }

    /// Not a very useful method
    pub fn downcast_mut<T: Payload>(&mut self) -> Option<&mut T> {
        let pld_box_dyn = &mut self.0;
        pld_box_dyn
            .as_any_mut()
            .downcast_mut::<MaybePayload<T>>()
            .map(|w| match w {
                MaybePayload::Nothing => unreachable!("invalid state"),
                MaybePayload::Just(e) => e,
            })
    }

    pub fn downcast<T: Payload>(mut self) -> Result<T, Self> {
        let pld_box_dyn = &mut self.0;
        let pld_dyn_mut = pld_box_dyn.as_mut();
        let pld_dyn_any = pld_dyn_mut.as_any_mut();
        if pld_dyn_any.is::<MaybePayload<T>>() {
            let pld_maybe_r = pld_dyn_any
                .downcast_mut::<MaybePayload<T>>()
                .expect("downcast success previously confirmed");
            let pld_maybe_v = replace(pld_maybe_r, MaybePayload::Nothing);
            Ok(pld_maybe_v.just().unwrap())
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

impl AsRef<dyn Payload> for BoxPayload {
    fn as_ref(&self) -> &dyn Payload {
        &self.0 as &dyn Payload
    }
}

// endregion:   --- BoxPayload

#[cfg(test)]
mod test {
    use crate::error::{BoxPayload, MaybePayload, PayloadPriv, Props};
    use std::any::TypeId;

    #[test]
    fn test_payload() {
        {
            fn f(payload: &dyn PayloadPriv) -> TypeId {
                payload.as_any().type_id()
            }

            let props = Props([].into());
            let id = props.as_any().type_id();

            println!("props type_id={:?}, f type_id={:?}", id, f(&props));
            assert_eq!(id, f(&props));
        }

        {
            fn f(payload: Props) -> TypeId {
                let boxed = BoxPayload::new(payload);
                boxed.0.as_ref().as_any().type_id()
            }

            let props1 = Props([].into());
            let props2 = Props([].into());
            let props3 = Props([].into());
            let id = MaybePayload::Just(props1).as_any().type_id();

            println!("props type_id={:?}, f type_id={:?}", id, f(props2));
            assert_eq!(id, f(props3));
        }
    }
}
