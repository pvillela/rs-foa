use super::common::AppCfgInfoArc;
use foa::{
    context::{Cfg, DbCtx},
    db::sqlx::pg::Db,
    error::FoaError,
    refinto::RefInto,
};
use sqlx::{Database, Executor, PgExecutor, Postgres, Transaction};
use tracing::instrument;

//=================
// This code section defines the stereotype signature

pub trait UpdateDaf<CTX>
where
    CTX: DbCtx<Db: Db>,
    for<'c> &'c mut <<CTX::Db as Db>::DB as Database>::Connection:
        Executor<'c, Database = <CTX::Db as Db>::DB>,
{
    #[allow(async_fn_in_trait)]
    async fn update_daf(
        age: i32,
        tx: &mut Transaction<'_, <CTX::Db as Db>::DB>,
    ) -> Result<(), FoaError<CTX>>;
}

//=================
// This code section implements the stereotype but depends on signatures only

pub struct UpdateDafCfgInfo<'a> {
    pub name: &'a str,
}

/// Trait alias
pub trait UpdateDafCtx: Cfg<CfgInfo: for<'a> RefInto<'a, UpdateDafCfgInfo<'a>>> {}
impl<CTX> UpdateDafCtx for CTX
where
    CTX: Cfg,
    CTX::CfgInfo: for<'a> RefInto<'a, UpdateDafCfgInfo<'a>>,
{
}

impl<CTX, T> UpdateDaf<CTX> for T
where
    CTX: UpdateDafCtx + DbCtx<Db: Db>,
    for<'c> &'c mut <<CTX::Db as Db>::DB as Database>::Connection:
        Executor<'c, Database = <CTX::Db as Db>::DB>,
{
    #[instrument(level = "trace", skip_all)]
    #[allow(async_fn_in_trait)]
    async fn update_daf(
        age: i32,
        tx: &mut Transaction<'_, <CTX::Db as Db>::DB>,
    ) -> Result<(), FoaError<CTX>> {
        let app_cfg_info = CTX::cfg();
        let cfg = app_cfg_info.ref_into();

        let res = sqlx::query("update users set age=$2 where name=$1;")
            .bind(cfg.name)
            .bind(age)
            .execute(&mut **tx)
            .await?;

        assert_eq!(res.rows_affected(), 1, "update_daf_boot");

        Ok(())
    }
}

//=================
// This code section depends on dependencies implementations

// *** N/A ***

//=================
// This code section depends on application configuration implementation

impl<'a> RefInto<'a, UpdateDafCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> UpdateDafCfgInfo {
        UpdateDafCfgInfo { name: &self.x }
    }
}

//=================
// This code section depends on platform stechnology-specific frameworks

// *** N/A ***
