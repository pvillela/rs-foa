//===========================
// region:      --- Payload

use std::any::Any;
use std::fmt::Debug;

pub trait Payload: Debug + Send + Sync + 'static {}

impl<T> Payload for T where T: Debug + Send + Sync + 'static {}

// endregion:   --- Payload

//===========================
// region:      --- BoxPayload

pub(crate) trait PayloadPriv: Payload {
    fn as_any(&self) -> &dyn Any;

    /// For exploratory purposes only
    #[cfg(test)]
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T> PayloadPriv for T
where
    T: Payload,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// For exploratory purposes only
    #[cfg(test)]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

pub struct BoxPayload(pub(crate) Box<dyn PayloadPriv>);

impl BoxPayload {
    pub fn new(inner: impl Payload) -> Self {
        Self(Box::new(inner))
    }

    pub fn is<T: Payload>(&self) -> bool {
        self.0.as_ref().as_any().is::<T>()
    }

    pub fn downcast_ref<T: Payload>(&self) -> Option<&T> {
        self.0.as_ref().as_any().downcast_ref::<T>()
    }

    /// For exploratory purposes only
    #[cfg(test)]
    pub fn downcast_mut<T: Payload>(&mut self) -> Option<&mut T> {
        self.0.as_mut().as_any_mut().downcast_mut::<T>()
    }

    pub fn downcast<T: Payload>(self) -> Result<Box<T>, Self> {
        if self.is::<T>() {
            let pld_box_any = self.0.into_any();
            let pld_box = pld_box_any
                .downcast::<T>()
                .expect("downcast success previously ensured");
            Ok(pld_box)
        } else {
            Err(self)
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
    use crate::error::foa_error::Props;
    use crate::error::{BoxPayload, PayloadPriv};
    use std::any::TypeId;

    fn make_props() -> Props {
        Props {
            pairs: [].into(),
            protected: false,
        }
    }

    #[test]
    fn test_payload() {
        let props = make_props();

        {
            fn f(payload: &dyn PayloadPriv) -> TypeId {
                payload.as_any().type_id()
            }

            let props = props.clone();
            let id = props.as_any().type_id();

            println!("props type_id={:?}, f type_id={:?}", id, f(&props));
            assert_eq!(id, f(&props));
        }

        {
            fn f(payload: Props) -> TypeId {
                let boxed = BoxPayload::new(payload);
                boxed.0.as_ref().as_any().type_id()
            }

            let props1 = props.clone();
            let props2 = props.clone();
            let props3 = props.clone();
            let id = props1.as_any().type_id();

            println!("props type_id={:?}, f type_id={:?}", id, f(props2));
            assert_eq!(id, f(props3));
        }
    }

    #[test]
    fn test_downcast_ref() {
        let props = make_props();

        let props1 = props.clone();
        let props2 = props.clone();

        let boxed = BoxPayload::new(props1);
        let downcast_ref = boxed.downcast_ref::<Props>().unwrap();
        assert_eq!(downcast_ref, &props2);
    }

    #[test]
    fn test_downcast_mut() {
        let props = make_props();

        let props1 = props.clone();
        let props2 = props.clone();

        let mut boxed = BoxPayload::new(props1);
        let downcast_mut = boxed.downcast_mut::<Props>().unwrap();
        assert_eq!(downcast_mut, &props2);
    }
}
