use std::error::Error as StdError;

pub fn extract_boxed_error<T: StdError + 'static>(
    boxed: Box<dyn StdError + Send + Sync>,
) -> Result<T, Box<dyn StdError + Send + Sync + 'static>> {
    let err_box = boxed.downcast::<T>()?;
    let err: T = *err_box;
    Ok(err)
}
