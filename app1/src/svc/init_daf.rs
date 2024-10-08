use crate::svc::common::AppCfgInfoArc;
use foa::{
    context::Cfg,
    db::sqlx::{AsyncTxFn, PgDbCtx},
    refinto::RefInto,
    Error, Result,
};
use sqlx::{Postgres, Transaction};
use tracing::instrument;

// region:      --- Stereotype signature

pub trait InitDaf<CTX> {
    #[allow(async_fn_in_trait)]
    async fn init_daf(tx: &mut Transaction<'_, Postgres>) -> Result<()>;
}

// endregion:   --- Stereotype signature

// region:      --- Stereotype implementation with dependencies' signatures only

pub struct InitDafCfgInfo<'a> {
    pub name: &'a str,
    pub initial_age: i32,
}

/// Trait alias
pub trait InitDafCtx: Cfg<CfgInfo: for<'a> RefInto<'a, InitDafCfgInfo<'a>>> {}
impl<CTX> InitDafCtx for CTX
where
    CTX: Cfg,
    CTX::CfgInfo: for<'a> RefInto<'a, InitDafCfgInfo<'a>>,
{
}

impl<CTX, T> InitDaf<CTX> for T
where
    CTX: InitDafCtx,
{
    #[instrument(level = "trace", skip_all)]
    #[allow(async_fn_in_trait)]
    async fn init_daf(tx: &mut Transaction<'_, Postgres>) -> Result<()> {
        let app_cfg_info = CTX::cfg();
        let cfg = app_cfg_info.ref_into();

        let _ = sqlx::query("delete from users where name = $1")
            .bind(cfg.name)
            .execute(&mut **tx)
            .await?;

        let res = sqlx::query(
            "insert into users (name, email, age)
             values ($1, $2, $3);",
        )
        .bind(cfg.name)
        .bind(cfg.name.to_owned() + "@example.com")
        .bind(cfg.initial_age)
        .execute(&mut **tx)
        .await?;

        assert_eq!(res.rows_affected(), 1, "init_daf_boot insert");

        Ok(())
    }
}

/// Stereotype instance
pub struct InitDafI<CTX: InitDafCtx>(pub CTX);

// endregion:   --- Stereotype implementation with dependencies' signatures only

// region:      --- Depends on dependencies' implementations

// *** N/A ***

// endregion:   --- Depends on dependencies' implementations

// region:      --- Additional platform technology-specific code

impl<CTX> AsyncTxFn for InitDafI<CTX>
where
    CTX: InitDafCtx + PgDbCtx + Sync + Send,
    CTX::CfgInfo: Send,
{
    type In = ();
    type Out = ();
    type E = Error;
    type Db = CTX::Db;

    async fn invoke(&self, _: (), tx: &mut Transaction<'_, Postgres>) -> Result<()> {
        <InitDafI<CTX> as InitDaf<CTX>>::init_daf(tx).await
    }
}

// endregion:   --- Additional platform technology-specific code

// region:      --- Depends on application configuration implementation

impl<'a> RefInto<'a, InitDafCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> InitDafCfgInfo {
        InitDafCfgInfo {
            name: &self.x,
            initial_age: self.y,
        }
    }
}

// endregion:   --- Depends on application configuration implementation
