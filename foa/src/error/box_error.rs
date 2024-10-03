use crate::error::utils;
use serde::Serialize;
use serde_json::Value;
use std::{
    any::Any,
    error::Error as StdError,
    fmt::{Debug, Display},
    mem::replace,
};

// region:      --- JserError

/// Trait for errors that can be serialized to JSON with [`serde_json`].
pub trait JserError: StdError + Send + Sync + 'static {
    fn to_json(&self) -> Value;
}

impl<T> JserError for T
where
    T: StdError + Serialize + Send + Sync + 'static,
{
    fn to_json(&self) -> Value {
        serde_json::to_value(self).expect("serde_json::to_value() error")
    }
}

impl StdError for Box<dyn JserError> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.as_ref().source()
    }
}

// endregion:   --- JserError

// region:      --- StdBoxError

pub struct StdBoxError(pub(crate) Box<dyn StdError + Send + Sync + 'static>);

impl StdBoxError {
    pub fn new(inner: impl StdError + Send + Sync + 'static) -> Self {
        Self(Box::new(inner))
    }

    pub fn as_dyn_std_error(&self) -> &(dyn StdError + 'static) {
        self.0.as_ref()
    }
}

impl Debug for StdBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for StdBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl StdError for StdBoxError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl Serialize for StdBoxError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut text = String::new();
        text.push_str("recursive_msg(");
        text.push_str(&utils::error_recursive_msg(self));
        text.push(')');
        serializer.serialize_str(&text)
    }
}

// endregion:   --- StdBoxError

// region:      --- JserBoxError

trait JserErrorPriv: JserError {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> JserErrorPriv for T
where
    T: JserError,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl StdError for Box<dyn JserErrorPriv> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.as_ref().source()
    }
}

enum ErrorWithNull<T: JserError> {
    Null,
    Real(T),
}

impl<T: JserError> ErrorWithNull<T> {
    fn real(self) -> Option<T> {
        match self {
            Self::Null => None,
            Self::Real(e) => Some(e),
        }
    }
}

impl<T: JserError> Debug for ErrorWithNull<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => f.write_str(""),
            Self::Real(e) => Debug::fmt(e, f),
        }
    }
}

impl<T: JserError> Display for ErrorWithNull<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => f.write_str(""),
            Self::Real(e) => Display::fmt(e, f),
        }
    }
}

impl<T: JserError> StdError for ErrorWithNull<T> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Null => None,
            Self::Real(e) => e.source(),
        }
    }
}

impl<T: JserError> JserError for ErrorWithNull<T> {
    fn to_json(&self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Real(e) => e.to_json(),
        }
    }
}

pub struct JserBoxError(Box<dyn JserErrorPriv>);

impl JserBoxError {
    pub fn new(inner: impl JserError) -> Self {
        Self(Box::new(ErrorWithNull::Real(inner)))
    }

    fn as_dyn_std_error(&self) -> &(dyn StdError + 'static) {
        &self.0 as &dyn StdError
    }

    pub fn downcast_ref<T: JserError>(&self) -> Option<&T> {
        let err_box_dyn = &self.0;
        err_box_dyn
            .as_any()
            .downcast_ref::<ErrorWithNull<T>>()
            .map(|w| match w {
                ErrorWithNull::Null => unreachable!("invalid state"),
                ErrorWithNull::Real(e) => e,
            })
    }

    /// Not a very useful method
    pub fn downcast_mut<T: JserError>(&mut self) -> Option<&mut T> {
        let err_box_dyn = &mut self.0;
        err_box_dyn
            .as_any_mut()
            .downcast_mut::<ErrorWithNull<T>>()
            .map(|w| match w {
                ErrorWithNull::Null => unreachable!("invalid state"),
                ErrorWithNull::Real(e) => e,
            })
    }

    pub fn downcast<T: JserError>(mut self) -> Result<T, Self> {
        let err_box_dyn = &mut self.0;
        let err_dyn_any = err_box_dyn.as_any_mut();
        if err_dyn_any.is::<ErrorWithNull<T>>() {
            let err_with_null_r = err_dyn_any
                .downcast_mut::<ErrorWithNull<T>>()
                .expect("downcast success previously confirmed");
            let err_with_null_v = replace(err_with_null_r, ErrorWithNull::Null);
            Ok(err_with_null_v.real().unwrap())
        } else {
            Err(self)
        }
    }

    /// If the boxed value is of type `T`, returns `Err(f(value))`; otherwise, returns `Ok(self)`.
    /// This unusual signature facilitates chaining of calls of this method with different types.
    pub fn with_downcast<T: JserError, U>(self, f: impl FnOnce(T) -> U) -> Result<Self, U> {
        let res = self.downcast::<T>();
        match res {
            Ok(t) => Err(f(t)),
            Err(err) => Ok(err),
        }
    }
}

impl Debug for JserBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for JserBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl StdError for JserBoxError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl Serialize for JserBoxError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_json().serialize(serializer)
    }
}

// endregion:   --- JserBoxError

// region:      --- BoxError

#[allow(private_interfaces)]
pub enum BoxError {
    Std(StdBoxError),
    Ser(JserBoxError),
}

impl BoxError {
    pub fn new_ser(inner: impl StdError + Serialize + Send + Sync + 'static) -> Self {
        Self::Ser(JserBoxError::new(inner))
    }

    pub fn new_std(inner: impl StdError + Send + Sync + 'static) -> Self {
        Self::Std(StdBoxError::new(inner))
    }

    pub fn as_dyn_std_error(&self) -> &(dyn StdError + 'static) {
        match self {
            Self::Std(err) => err.as_dyn_std_error(),
            Self::Ser(err) => err.as_dyn_std_error(),
        }
    }
}

impl Debug for BoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Std(err) => Debug::fmt(err, f),
            Self::Ser(err) => Debug::fmt(err, f),
        }
    }
}

impl Display for BoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Std(err) => Display::fmt(err, f),
            Self::Ser(err) => Display::fmt(err, f),
        }
    }
}

impl StdError for BoxError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Std(err) => err.source(),
            Self::Ser(err) => err.source(),
        }
    }
}

impl Serialize for BoxError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Std(err) => err.serialize(serializer),
            Self::Ser(err) => err.serialize(serializer),
        }
    }
}

// endregion:   --- BoxError

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::{PropsError, TrivialError};

    #[test]
    fn test_downcast() {
        let e = TrivialError("e");
        let jsb_e = JserBoxError::new(e);

        let jsb_e = jsb_e
            .downcast::<PropsError>()
            .expect_err("downcast should fail");

        let e_d: TrivialError = jsb_e.downcast().expect("downcast should succeed");
        assert_eq!(e_d.to_string(), "e".to_owned());
    }

    #[test]
    fn test_downcast_ref() {
        let e = TrivialError("e");
        let jsb_e = JserBoxError::new(e);

        let e_d_opt = jsb_e.downcast_ref::<PropsError>();
        assert!(e_d_opt.is_none(), "downcast should fail");

        let e_d: TrivialError = jsb_e.downcast().expect("downcast should succeed");
        assert_eq!(e_d.to_string(), "e".to_owned());
    }

    #[test]
    fn test_downcast_mut() {
        let e = TrivialError("e");
        let mut jsb_e = JserBoxError::new(e);

        let e_d_opt = jsb_e.downcast_mut::<PropsError>();
        assert!(e_d_opt.is_none(), "downcast should fail");

        let e_d: TrivialError = jsb_e.downcast().expect("downcast should succeed");
        assert_eq!(e_d.to_string(), "e".to_owned());
    }
}
