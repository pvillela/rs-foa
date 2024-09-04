use super::common::AppCfgInfoArc;
use foa::{
    context::{Cfg, Itself},
    db::sqlx::{AsyncTxFn, PgDbCtx},
    error::FoaError,
    refinto::RefInto,
};
use sqlx::{Postgres, Transaction};
use std::marker::PhantomData;
use tracing::instrument;

//=================
// This section defines the stereotype signature

pub trait InitDaf<CTX> {
    #[allow(async_fn_in_trait)]
    async fn init_daf(tx: &mut Transaction<'_, Postgres>) -> Result<(), FoaError<CTX>>;
}

//=================
// This section implements the stereotype but depends on signatures only

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
    async fn init_daf(tx: &mut Transaction<'_, Postgres>) -> Result<(), FoaError<CTX>> {
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
pub struct InitDafI<CTX: InitDafCtx>(PhantomData<CTX>);

impl<CTX: InitDafCtx> Itself for InitDafI<CTX> {
    fn it() -> Self {
        InitDafI(PhantomData)
    }
}

//=================
// This section depends on dependencies implementations

// *** N/A ***

//=================
// This section depends on application configuration implementation

impl<'a> RefInto<'a, InitDafCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> InitDafCfgInfo {
        InitDafCfgInfo {
            name: &self.x,
            initial_age: self.y,
        }
    }
}

//=================
// This section has additional platform technology-specific code

impl<CTX> AsyncTxFn<CTX> for InitDafI<CTX>
where
    CTX: InitDafCtx + PgDbCtx + Sync,
    CTX::CfgInfo: Send,
{
    type In = ();
    type Out = ();
    type E = FoaError<CTX>;

    async fn invoke(&self, _: (), tx: &mut Transaction<'_, Postgres>) -> Result<(), FoaError<CTX>> {
        InitDafI::<CTX>::init_daf(tx).await
    }
}
