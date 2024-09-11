use std::i32;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppCfgInfo {
    pub x: String,
    pub y: i32,
    pub z: i32,
    pub refresh_count: u32,
}

pub type AppCfgInfoArc = Arc<AppCfgInfo>;
