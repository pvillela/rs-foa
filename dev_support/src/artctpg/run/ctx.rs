use crate::artctpg::svc::common::{AppCfgInfo, AppCfgInfoArc};
use crate::artctpg::svc::InitDafI;
use arc_swap::ArcSwap;
use axum::http::request::Parts;
use foa::context::ErrCtx;
use foa::{
    context::{Cfg, Locale, LocaleCtx},
    db::sqlx::{invoke_in_tx, Db, DbCtx},
    static_state::StaticStateMut,
    tokio::{
        task_local::{TaskLocal, TaskLocalCtx},
        task_local_ext::locale_from_task_local,
    },
    Error,
};
use sqlx::{Pool, Postgres};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, OnceLock,
};
use tokio::task::LocalKey;

static CTX_INFO: OnceLock<ArcSwap<CtxInfo>> = OnceLock::new();
static REFRESH_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub struct CtxInfo {
    app_cfg: AppCfgInfoArc,
    db: Pool<Postgres>,
}

pub async fn new_db_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    Pool::connect("postgres://testuser:testpassword@localhost:9999/testdb").await
}

#[derive(Debug, Clone)]
pub struct Ctx;

pub struct SubCtx;

impl ErrCtx for Ctx {
    type LocalizedMsg = ();
}

impl StaticStateMut for Ctx {
    type State = CtxInfo;

    fn get_static() -> &'static OnceLock<ArcSwap<CtxInfo>> {
        &CTX_INFO
    }
}

impl Cfg for Ctx {
    type CfgInfo = AppCfgInfoArc;

    fn cfg() -> Self::CfgInfo {
        Ctx::state().app_cfg.clone()
    }
}

impl Db for SubCtx {
    type Database = Postgres;

    async fn pool() -> Result<Pool<Postgres>, sqlx::Error> {
        Ok(Ctx::state().db.clone())
    }
}

impl DbCtx for Ctx {
    type Db = SubCtx;
}

impl Ctx {
    /// Initializes context.
    ///
    /// # Panics
    /// If there are any errors during initialization.
    pub async fn init() {
        Self::refresh_cfg()
            .await
            .expect("Ctx::init: read_app_cfg_info error");
        new_db_pool().await.expect("Ctx::init: db_pool error");
        invoke_in_tx(&InitDafI(Ctx), ())
            .await
            .expect("Ctx::init: data initialization error");
    }

    /// Simulates reading [`AppCfgInfo`] from external source.
    pub async fn read_app_cfg_info() -> Result<AppCfgInfo, Error<Ctx>> {
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
        Ok(app_cfg_info)
    }

    /// Refreshes [`CtxInfo`] based on [`AppCfgInfo`] read from external source.
    pub async fn refresh_cfg() -> Result<(), Error<Ctx>> {
        let app_cfg = Arc::new(Self::read_app_cfg_info().await?);
        let new_state = match Self::try_state() {
            None => CtxInfo {
                app_cfg,
                db: new_db_pool().await?,
            },
            Some(state) => CtxInfo {
                app_cfg,
                db: state.db.clone(),
            },
        };
        Ctx::update_state(new_state);
        Ok(())
    }
}

tokio::task_local! {
    static CTX_TL: Parts;
}

impl TaskLocal for SubCtx {
    type Value = Parts;

    fn local_key() -> &'static LocalKey<Self::Value> {
        &CTX_TL
    }
}

impl Locale for SubCtx {
    fn locale() -> impl std::ops::Deref<Target = str> {
        locale_from_task_local::<Self>("en-CA")
    }
}

impl TaskLocalCtx for Ctx {
    type TaskLocal = SubCtx;
}

impl LocaleCtx for Ctx {
    type Locale = SubCtx;
}
