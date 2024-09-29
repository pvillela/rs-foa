use std::error::Error as StdError;

pub fn extract_boxed_error<T: StdError + 'static>(boxed: Box<dyn StdError>) -> Option<T> {
    let err_box = boxed.downcast::<T>().ok()?;
    let err: T = *err_box;
    Some(err)
}
