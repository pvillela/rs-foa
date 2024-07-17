use serde::Serialize;
use serde_json::Value;
use std::error::Error as StdError;

pub trait SerError: StdError {
    fn to_json(&self) -> Value;
}

impl StdError for Box<dyn SerError> {}

impl<T> SerError for T
where
    T: StdError + Serialize,
{
    fn to_json(&self) -> Value {
        serde_json::to_value(self).expect("serde_json::to_value() error")
    }
}

// impl Serialize for Box<dyn SerError> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         self.to_json().serialize(serializer)
//     }
// }
