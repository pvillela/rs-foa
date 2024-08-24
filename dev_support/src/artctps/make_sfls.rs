use super::{
    common::{AppCfgInfo, AppCfgInfoArc, AppErr},
    FooIn, FooOut, FooSflI,
};
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
    async fn pool_tx<'c>(&'c self) -> Result<Transaction<'c, Postgres>, SqlxError> {
        let pool =
            PgPool::connect("postgres://testuser:testpassword@localhost:9999/testdb").await?;
        pool.begin().await
    }
}

impl Itself<Ctx> for Ctx {
    fn itself() -> Ctx {
        Ctx
    }
}

pub async fn foo_sfl(input: FooIn) -> Result<FooOut, FoaError<Ctx>> {
    FooSflI::<Ctx>::exec_with_transaction(input).await
}
