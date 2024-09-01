use arc_swap::ArcSwap;
use foa::context::Cfg;
use foa::db::sqlx::{AsyncTxFn, Db};
use foa::error::FoaError;
use foa::static_state::{StaticState, StaticStateMut};
use sqlx::{Pool, Postgres};
use std::i32;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, OnceLock,
};

use crate::artctps::InitDafI;

static APP_CFG_INFO: OnceLock<ArcSwap<AppCfgInfo>> = OnceLock::new();
static CTX_INFO: OnceLock<CtxInfo> = OnceLock::new();
static REFRESH_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub struct AppCfgInfo {
    pub x: String,
    pub y: i32,
    pub z: i32,
    pub refresh_count: u32,
}

pub type AppCfgInfoArc = Arc<AppCfgInfo>;

#[derive(Debug, Clone)]
pub struct CtxInfo {
    db: Pool<Postgres>,
}

pub async fn db_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    let f = || async {
        let pool = Pool::connect("postgres://testuser:testpassword@localhost:9999/testdb").await?;
        Ok(CtxInfo { db: pool })
    };
    <Ctx as StaticState<_>>::get_or_init_state_async(f)
        .await
        .map(|info| info.db.clone())
}

#[derive(Debug, Clone)]
pub struct Ctx;

impl StaticState<CtxInfo> for Ctx {
    fn get_static() -> &'static OnceLock<CtxInfo> {
        &CTX_INFO
    }
}

impl StaticStateMut<AppCfgInfo> for Ctx {
    fn get_static() -> &'static OnceLock<ArcSwap<AppCfgInfo>> {
        &APP_CFG_INFO
    }
}

impl Cfg for Ctx {
    type CfgInfo = AppCfgInfoArc;

    fn cfg() -> Self::CfgInfo {
        <Ctx as StaticStateMut<_>>::state()
    }
}

impl Db for Ctx {
    type Database = Postgres;

    async fn pool() -> Result<Pool<Postgres>, sqlx::Error> {
        Ok(<Ctx as StaticState<_>>::state().db.clone())
    }
}

impl Ctx {
    /// Initializes context.
    ///
    /// # Panics
    /// If there are any errors during initialization.
    pub async fn init() {
        Self::refresh_app_cfg_info()
            .await
            .expect("Ctx::init: read_app_cfg_info error");
        db_pool().await.expect("Ctx::init: db_pool error");
        InitDafI::<Ctx>::in_tx(())
            .await
            .expect("Ctx::init: data initialization error");
    }

    /// Simulates reading app config info from external source.
    pub async fn refresh_app_cfg_info() -> Result<(), FoaError<Ctx>> {
        let f = || async {
            let count = REFRESH_COUNT.fetch_add(1, Ordering::Relaxed);
            let app_cfg_info = match REFRESH_COUNT.load(Ordering::Relaxed) {
                0 => AppCfgInfo {
                    x: "Paulo".into(),
                    y: 10,
                    z: 2,
                    refresh_count: count,
                },
                _ => AppCfgInfo {
                    x: "Paulo".into(),
                    y: 100,
                    z: 20,
                    refresh_count: count,
                },
            };
            Ok::<AppCfgInfo, FoaError<Ctx>>(app_cfg_info)
        };
        let app_cfg_info = f().await?;
        <Ctx as StaticStateMut<_>>::update_state(app_cfg_info);
        Ok(())
    }
}
