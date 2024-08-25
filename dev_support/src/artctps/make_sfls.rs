use super::common::{AppCfgInfo, AppCfgInfoArc};
use foa::{
    appcfg::AppCfg,
    context::{Cfg, CfgCtx},
    db::sqlx::pg::{Db, Itself},
};
use sqlx::{PgPool, Postgres, Transaction};

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

impl Db for Ctx {
    async fn pool_tx<'c>(&'c self) -> Result<Transaction<'c, Postgres>, sqlx::Error> {
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
