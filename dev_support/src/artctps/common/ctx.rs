use arc_swap::{ArcSwap, ArcSwapAny};
use foa::context::{Context, RefCntWrapper};
use foa::{context::CfgCtx, db::sqlx::pg::Db};
use sqlx::PgPool;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, OnceLock,
};

static CTX_INFO: OnceLock<ArcSwap<Ctx0>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct AppCfgInfo {
    pub x: String,
    pub y: i32,
    pub z: bool,
}

impl AppCfgInfo {
    pub fn refresh_app_configuration() {
        let count = REFRESH_COUNT.fetch_add(1, Ordering::Relaxed);
        let cfg_info = AppCfgInfo {
            x: format!("refreshed-{}", count),
            y: 1042,
            z: true,
        };
        Ctx::refresh_app_cfg(cfg_info.into());
    }
}

#[derive(Debug, Clone)]
pub struct Ctx0 {
    cfg: AppCfgInfoArc,
    db: PgPool,
}

#[derive(Debug, Clone)]
pub struct Ctx(Arc<Ctx0>);

impl RefCntWrapper for Ctx {
    type Inner = Arc<Ctx0>;

    fn wrap(inner: Self::Inner) -> Self {
        Ctx(inner)
    }

    fn inner(&self) -> Self::Inner {
        self.0.clone()
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
                x: "initial".to_owned(),
                y: 42,
                z: false,
            }
        };

        let pool = {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime error")
                .block_on(async {
                    PgPool::connect("postgres://testuser:testpassword@localhost:9999/testdb")
                        .await
                        .expect("unable to connect to Postgres")
                })
        };

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

impl CfgCtx for Ctx {
    type Cfg = Ctx;
}

pub async fn db_pool() -> Result<PgPool, sqlx::Error> {
    PgPool::connect("postgres://testuser:testpassword@localhost:9999/testdb").await
}

impl Db for Ctx {
    async fn pool() -> Result<PgPool, sqlx::Error> {
        db_pool().await
    }
}

pub type AppCfgInfoArc = Arc<AppCfgInfo>;

static REFRESH_COUNT: AtomicU32 = AtomicU32::new(0);
