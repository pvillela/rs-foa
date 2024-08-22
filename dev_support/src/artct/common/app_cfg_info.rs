use crate::artct::common::{DbCfg, DummyDbPool};
use arc_swap::ArcSwap;
use foa::appcfg::AppCfg;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, OnceLock,
};

#[derive(Debug, Clone)]
pub struct AppCfgInfo {
    pub x: String,
    pub y: i32,
    pub z: bool,
}

pub type AppCfgInfoArc = Arc<AppCfgInfo>;

static APP_CONFIGURATION: OnceLock<ArcSwap<AppCfgInfo>> = OnceLock::new();

#[allow(unused)]
static REFRESH_COUNT: AtomicU32 = AtomicU32::new(0);

impl AppCfg for AppCfgInfo {
    fn app_cfg_static() -> &'static OnceLock<ArcSwap<Self>> {
        &APP_CONFIGURATION
    }

    fn app_config_info() -> Self {
        if REFRESH_COUNT.load(Ordering::Relaxed) == 0 {
            REFRESH_COUNT.store(1, Ordering::Relaxed);
            AppCfgInfo {
                x: "initial".to_owned(),
                y: 42,
                z: false,
            }
        } else {
            let count = REFRESH_COUNT.fetch_add(1, Ordering::Relaxed);
            AppCfgInfo {
                x: format!("refreshed-{}", count),
                y: 1042,
                z: true,
            }
        }
    }
}

impl DbCfg for AppCfgInfoArc {
    fn get_pool(&self) -> &DummyDbPool {
        // TODO: implement this properly
        static POOL: OnceLock<DummyDbPool> = OnceLock::new();
        POOL.get_or_init(|| DummyDbPool)
    }
}

pub fn get_pool() -> &'static DummyDbPool {
    static CFG: OnceLock<AppCfgInfoArc> = OnceLock::new();
    let cfg = CFG.get_or_init(|| AppCfgInfo::get_app_configuration().clone());
    cfg.get_pool()
}
