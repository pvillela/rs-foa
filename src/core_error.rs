use crate::{interpolated_localized_msg, ErrCtx, NoDebug};
use std::{error::Error as StdError, fmt::Debug, marker::PhantomData, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[error("{}", interpolated_localized_msg::<CTX>(kind, args))]
pub struct CoreError<CTX>
where
    CTX: ErrCtx,
{
    pub kind: &'static str,
    pub args: Vec<String>,
    pub source: Option<Arc<dyn StdError>>,
    _ctx: NoDebug<PhantomData<CTX>>,
}

impl<CTX> CoreError<CTX>
where
    CTX: ErrCtx,
{
    pub fn new(kind: &'static str, args: Vec<String>) -> Self {
        CoreError {
            kind,
            args,
            source: None,
            _ctx: NoDebug(PhantomData),
        }
    }

    pub fn new_with_source(
        kind: &'static str,
        args: Vec<String>,
        source: impl StdError + 'static,
    ) -> Self {
        CoreError {
            kind,
            args,
            source: Some(Arc::new(source)),
            _ctx: NoDebug(PhantomData),
        }
    }
}
