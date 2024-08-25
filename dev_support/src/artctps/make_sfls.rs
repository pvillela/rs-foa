use super::{
    common::{AppCfgInfo, AppCfgInfoArc, AppErr},
    FooIn, FooOut, FooSfl, FooSflI,
};
use axum::extract::Extension;
use foa::{
    appcfg::AppCfg,
    context::{Cfg, CfgCtx},
    db::sqlx::pg::{AsyncFnTx, Db, Itself},
    error::FoaError,
};
use sqlx::{Error as SqlxError, PgPool, Postgres, Transaction};

#[derive(Debug)]
pub struct Ctx;

pub struct CtxCfg;

impl Cfg for CtxCfg {
    type Info = AppCfgInfoArc;

    fn cfg() -> Self::Info {
        AppCfgInfo::get_app_configuration()
    }
}

impl CfgCtx for Ctx {
    type Cfg = CtxCfg;
}

// struct CtxDbClient;

// impl DbClientDefault for CtxDbClient {}

// impl DbCtx for Ctx {
//     type DbClient = CtxDbClient;
// }

// struct Ctx;

impl Db for Ctx {
    async fn pool_tx<'c, CTX>(&'c self) -> Result<Transaction<'c, Postgres>, FoaError<CTX>> {
        let pool =
            PgPool::connect("postgres://testuser:testpassword@localhost:9999/testdb").await?;
        pool.begin().await.map_err(|err| err.into())
    }
}

impl Itself<Ctx> for Ctx {
    fn itself() -> Ctx {
        Ctx
    }
}

#[derive(Clone)]
pub struct ApiContext {
    // config: Arc<Config>,
    db: PgPool,
}

pub async fn foo_sfl(input: FooIn) -> Result<FooOut, FoaError<Ctx>> {
    FooSflI::<Ctx>::exec_with_transaction(input).await
}

pub async fn foo_sfl1(input: FooIn, ctx: Extension<ApiContext>) -> Result<FooOut, ()> {
    let mut tx = ctx.db.begin().await.map_err(|_| ())?;
    return Err(());
    let res = FooSflI::<Ctx>::foo_sfl(input, &mut tx).await;
    if res.is_ok() {
        tx.commit().await.map_err(|_| ())?;
    } else {
        tx.rollback().await.map_err(|_| ())?;
    }
    res.map_err(|_| ())
}
