use crate::{display_interpolated, NoDebug};
use std::{
    collections::HashMap, error::Error as StdError, fmt::Debug, marker::PhantomData, sync::Arc,
};
use thiserror::Error;

fn display_msg(display_map: &HashMap<&str, &str>, display_key: &str, args: &Vec<String>) -> String {
    let raw_msg = display_map.get(display_key);
    let Some(raw_msg) = raw_msg else {
        return "invalid error key".to_owned();
    };
    display_interpolated(raw_msg, args)
}

pub trait ErrorDisplayMap {
    fn display_map<'a>() -> &'a HashMap<&'a str, &'a str>;
}

pub trait Locale {
    fn locale<'a>() -> &'a str;
}

#[derive(Error, Debug, Clone)]
#[error("{}", display_msg(CTX::display_map(), &self.display_key(), args))]
pub struct CoreError<CTX>
where
    CTX: ErrorDisplayMap + Locale,
{
    pub kind: &'static str,
    pub args: Vec<String>,
    pub source: Option<Arc<dyn StdError>>,
    _ctx: NoDebug<PhantomData<CTX>>,
}

impl<CTX> CoreError<CTX>
where
    CTX: ErrorDisplayMap + Locale,
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

    fn display_key(&self) -> String {
        self.kind.to_owned() + "-" + CTX::locale()
    }
}
