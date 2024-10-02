use std::error::Error as StdError;

pub fn extract_boxed_error<T: StdError + 'static>(
    boxed: Box<dyn StdError + Send + Sync>,
) -> Result<T, Box<dyn StdError + Send + Sync + 'static>> {
    let err_box = boxed.downcast::<T>()?;
    let err: T = *err_box;
    Ok(err)
}

pub fn swap_result<T, E>(f: impl FnOnce() -> std::result::Result<E, T>) -> Result<T, E> {
    match f() {
        Ok(e) => Err(e),
        Err(t) => Ok(t),
    }
}
