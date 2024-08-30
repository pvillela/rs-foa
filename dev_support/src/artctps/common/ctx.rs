use arc_swap::{ArcSwap, ArcSwapAny};
use foa::context::{Context, Itself, RefCntWrapper};
use foa::db::sqlx::{AsyncTxFn, Db};
use sqlx::{Pool, Postgres};
use std::i32;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, OnceLock,
};

use crate::artctps::InitDafI;

static CTX_INFO: OnceLock<ArcSwap<Ctx0>> = OnceLock::new();
static DB_POOL: OnceLock<Pool<Postgres>> = OnceLock::new();
static REFRESH_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub struct AppCfgInfo {
    pub x: String,
    pub y: i32,
    pub z: i32,
    pub refresh_count: u32,
}

pub type AppCfgInfoArc = Arc<AppCfgInfo>;

impl AppCfgInfo {
    pub fn refresh_app_configuration() {
        let count = REFRESH_COUNT.fetch_add(1, Ordering::Relaxed);
        let cfg_info = AppCfgInfo {
            x: "Paulo".into(),
            y: 100,
            z: 20,
            refresh_count: count,
        };
        Ctx::refresh_app_cfg(cfg_info.into());
    }
}

#[derive(Debug, Clone)]
pub struct Ctx0 {
    cfg: AppCfgInfoArc,
    db: Pool<Postgres>,
}

#[derive(Debug, Clone)]
pub struct Ctx(Arc<Ctx0>);

impl Db for Ctx {
    type Database = Postgres;

    async fn pool() -> Result<Pool<Postgres>, sqlx::Error> {
        Ok(Ctx::itself().0.db.clone())
    }
}

impl Ctx {
    /// Initializes context.
    ///
    /// # Panics
    /// If there are any errors during initialization.
    pub async fn init() {
        db_pool().await.expect("Ctx::init: db_pool error");
        InitDafI::<Ctx>::in_tx(())
            .await
            .expect("Ctx::init: data initialization error");
    }
}

impl RefCntWrapper for Ctx {
    type Inner = Arc<Ctx0>;

    fn wrap(inner: Self::Inner) -> Self {
        Ctx(inner)
    }

    fn inner(&self) -> Self::Inner {
        self.0.clone()
    }
}

pub async fn db_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    match DB_POOL.get() {
        Some(db_pool) => Ok(db_pool.clone()),
        None => {
            let pool =
                Pool::connect("postgres://testuser:testpassword@localhost:9999/testdb").await?;
            Ok(DB_POOL.get_or_init(|| pool).clone())
        }
    }
}

impl Context for Ctx {
    type CfgInfo = AppCfgInfoArc;

    fn ctx_static() -> &'static OnceLock<ArcSwapAny<Self::Inner>> {
        &CTX_INFO
    }

    fn new_inner() -> Self::Inner {
        let app_cfg = {
            REFRESH_COUNT.store(1, Ordering::Relaxed);
            AppCfgInfo {
                x: "Paulo".into(),
                y: 10,
                z: 2,
                refresh_count: 1,
            }
        };

        let pool = DB_POOL
            .get()
            .expect("DB_POOL should be initialized")
            .clone();

        Ctx0 {
            cfg: app_cfg.into(),
            db: pool,
        }
        .into()
    }

    fn inner_with_updated_app_cfg(inner: &Self::Inner, cfg_info: Self::CfgInfo) -> Self::Inner {
        let _ = REFRESH_COUNT.fetch_add(1, Ordering::Relaxed);
        let cfg = cfg_info;
        let db = inner.db.clone();
        Ctx0 { cfg, db }.into()
    }

    fn get_app_cfg(&self) -> Self::CfgInfo {
        self.0.cfg.clone()
    }
}
