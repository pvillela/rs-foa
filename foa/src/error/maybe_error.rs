use super::NullError;
use std::{
    any::Any,
    error::Error as StdError,
    fmt::{Debug, Display},
};

// ==================================
// region:      --- MaybeStdError

enum MaybeStdError<T: StdError + Send + Sync + 'static> {
    Nothing,
    Just(T),
}

impl<T: StdError + Send + Sync + 'static> MaybeStdError<T> {
    fn just(self) -> Option<T> {
        match self {
            Self::Nothing => None,
            Self::Just(e) => Some(e),
        }
    }

    fn just_ref(&self) -> Option<&T> {
        match self {
            Self::Nothing => None,
            Self::Just(e) => Some(e),
        }
    }
}

impl<T: StdError + Send + Sync + 'static> Debug for MaybeStdError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nothing => f.write_str(""),
            Self::Just(e) => Debug::fmt(e, f),
        }
    }
}

impl<T: StdError + Send + Sync + 'static> Display for MaybeStdError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nothing => f.write_str(""),
            Self::Just(e) => Display::fmt(e, f),
        }
    }
}

impl<T: StdError + Send + Sync + 'static> StdError for MaybeStdError<T> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Nothing => None,
            Self::Just(e) => e.source(),
        }
    }
}

// endregion:   --- MaybeStdError

// ==================================
// region:      --- MaybeStdBoxError

trait SsssError: StdError + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;

    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T> SsssError for T
where
    T: StdError + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl StdError for Box<dyn SsssError> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.as_ref().source()
    }
}

pub struct MaybeStdBoxError(Box<dyn SsssError>);

impl MaybeStdBoxError {
    pub fn new(inner: impl StdError + Send + Sync + 'static) -> Self {
        Self(Box::new(MaybeStdError::Just(inner)))
    }

    pub fn nothing() -> Self {
        Self(Box::new(MaybeStdError::<NullError>::Nothing))
    }

    pub fn is<T: StdError + Send + Sync + 'static>(&self) -> bool {
        self.0.as_ref().as_any().is::<MaybeStdError<T>>()
    }

    pub fn downcast_ref<T: StdError + Send + Sync + 'static>(&self) -> Option<&T> {
        let x = self.0.as_ref().as_any().downcast_ref::<MaybeStdError<T>>();
        match x {
            Some(y) => y.just_ref(),
            None => None,
        }
    }

    pub fn downcast<T: StdError + Send + Sync + 'static>(self) -> Result<T, Self> {
        if self.is::<T>() {
            let inner_box_any = self.0.into_any();
            let inner_box = inner_box_any
                .downcast::<MaybeStdError<T>>()
                .expect("downcast success previously ensured");
            Ok(inner_box.just().unwrap())
        } else {
            Err(self)
        }
    }
}

impl Debug for MaybeStdBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for MaybeStdBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl StdError for MaybeStdBoxError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

// endregion:   --- MaybeStdBoxError

#[cfg(test)]
mod test {
    use super::MaybeStdBoxError;
    use crate::error::TrivialError;

    #[test]
    fn test_downcast_ref() {
        {
            let triv_err = TrivialError("dummy");
            let maybe_err = MaybeStdBoxError::new(triv_err.clone());
            let ext_err_ref = maybe_err.downcast_ref::<TrivialError>();
            println!("{ext_err_ref:?}");
            assert_eq!(Some(&triv_err), ext_err_ref);
        }

        {
            let maybe_err = MaybeStdBoxError::nothing();
            let ext_err_ref = maybe_err.downcast_ref::<TrivialError>();
            println!("{ext_err_ref:?}");
            assert_eq!(None, ext_err_ref);
        }
    }

    #[test]
    fn test_downcast() {
        {
            let triv_err = TrivialError("dummy");
            let maybe_err = MaybeStdBoxError::new(triv_err.clone());
            let ext_err_ref = maybe_err.downcast::<TrivialError>();
            println!("{ext_err_ref:?}");
            assert_eq!(triv_err, ext_err_ref.unwrap());
        }

        {
            let maybe_err = MaybeStdBoxError::nothing();
            let ext_err_ref = maybe_err.downcast::<TrivialError>();
            println!("{ext_err_ref:?}");
            assert!(ext_err_ref.is_err());
        }
    }
}
