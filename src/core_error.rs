use std::{collections::HashMap, error::Error as StdError, fmt::Debug, sync::Arc};
use thiserror::Error;

use crate::{display_interpolated, NoDebug};

fn display_msg(display_map: &HashMap<&str, &str>, display_key: &str, args: &Vec<String>) -> String {
    let raw_msg = display_map.get(display_key);
    let Some(raw_msg) = raw_msg else {
        return "invalid error key".to_owned();
    };
    display_interpolated(raw_msg, args)
}

#[derive(Error, Debug, Clone)]
#[error("{}", display_msg(display_map, kind, args))]
pub struct CoreError {
    pub kind: &'static str,
    pub display_map: NoDebug<&'static HashMap<&'static str, &'static str>>,
    pub args: Vec<String>,
    pub source: Option<Arc<dyn StdError>>,
}
