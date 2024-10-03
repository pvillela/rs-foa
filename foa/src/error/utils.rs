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

struct It<'a> {
    curr_source: Option<&'a dyn StdError>,
}

impl<'a> Iterator for It<'a> {
    type Item = &'a dyn StdError;

    fn next(&mut self) -> Option<Self::Item> {
        match self.curr_source {
            None => None,
            Some(err) => {
                let next_src = err.source();
                self.curr_source = next_src;
                Some(err)
            }
        }
    }
}

pub fn error_chain(err: &dyn StdError) -> impl Iterator<Item = &dyn StdError> {
    It {
        curr_source: Some(err),
    }
}

pub fn error_recursive_msg(err: &(dyn StdError)) -> String {
    let mut chain_iter = error_chain(err);
    let mut buf = String::new();
    let mut closing_buf = String::new();

    let first = chain_iter
        .next()
        .expect("error chain always has a first element");
    buf.push_str(&first.to_string());

    for item in chain_iter {
        buf.push_str(", source_msg=[");
        buf.push_str(&item.to_string());
        closing_buf.push(']');
    }

    buf.push_str(&closing_buf);

    buf
}
