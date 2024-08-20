use super::{DbCfg, DummyDbPool};
use arc_swap::{ArcSwap, ArcSwapAny};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, OnceLock,
};

#[derive(Debug, Clone)]
pub struct AppCfgInfo0 {
    pub x: String,
    pub y: i32,
    pub z: bool,
}

pub type AppCfgInfo = Arc<AppCfgInfo0>;

static APP_CONFIGURATION: OnceLock<ArcSwap<AppCfgInfo0>> = OnceLock::new();

#[allow(unused)]
static REFRESH_COUNT: AtomicU32 = AtomicU32::new(0);

// Produce simulated initial APP_CONFIGURATION
fn initial_app_configuration() -> AppCfgInfo0 {
    AppCfgInfo0 {
        x: "initial".to_owned(),
        y: 42,
        z: false,
    }
}

fn get_app_config_arcswap() -> &'static ArcSwapAny<AppCfgInfo> {
    APP_CONFIGURATION.get_or_init(|| ArcSwap::from_pointee(initial_app_configuration()))
}

#[allow(unused)]
// Simulates initialization of APP_CONFIGURATION
pub fn initialize_app_configuration() {
    REFRESH_COUNT.store(0, Ordering::Relaxed);
    let cfg_as = get_app_config_arcswap();
    cfg_as.store(Arc::new(initial_app_configuration()));
}

#[allow(unused)]
// Simulates refresh of APP_CONFIGURATION
pub fn refresh_app_configuration() {
    let count = REFRESH_COUNT.fetch_add(1, Ordering::Relaxed);
    let cfg_as = get_app_config_arcswap();
    cfg_as.store(Arc::new(AppCfgInfo0 {
        x: format!("refreshed-{}", count),
        y: 1042,
        z: true,
    }));
}

pub fn get_app_configuration() -> AppCfgInfo {
    // println!("get_app_configuration has been called");
    let cfg_as = get_app_config_arcswap();
    cfg_as.load().clone()
}

impl DbCfg for AppCfgInfo {
    fn get_pool(&self) -> &DummyDbPool {
        // TODO: implement this properly
        static POOL: OnceLock<DummyDbPool> = OnceLock::new();
        POOL.get_or_init(|| DummyDbPool)
    }
}

pub fn get_pool() -> &'static DummyDbPool {
    static CFG: OnceLock<AppCfgInfo> = OnceLock::new();
    let cfg = CFG.get_or_init(|| get_app_configuration().clone());
    cfg.get_pool()
}
