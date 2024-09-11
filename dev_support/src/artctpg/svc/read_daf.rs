use crate::artctpg::svc::common::AppCfgInfoArc;
use foa::{context::Cfg, error::FoaError, refinto::RefInto};
use sqlx::{Postgres, Transaction};
use tracing::instrument;

//=================
// This section defines the stereotype signature

pub trait ReadDaf<CTX> {
    #[allow(async_fn_in_trait)]
    async fn read_daf(tx: &mut Transaction<'_, Postgres>) -> Result<i32, FoaError<CTX>>;
}

//=================
// This section implements the stereotype but depends on signatures only

pub struct ReadDafCfgInfo<'a> {
    pub name: &'a str,
}

/// Trait alias
pub trait ReadDafCtx: Cfg<CfgInfo: for<'a> RefInto<'a, ReadDafCfgInfo<'a>>> {}
impl<CTX> ReadDafCtx for CTX
where
    CTX: Cfg,
    CTX::CfgInfo: for<'a> RefInto<'a, ReadDafCfgInfo<'a>>,
{
}

impl<CTX, T> ReadDaf<CTX> for T
where
    CTX: ReadDafCtx,
{
    #[instrument(level = "trace", skip_all)]
    #[allow(async_fn_in_trait)]
    async fn read_daf(tx: &mut Transaction<'_, Postgres>) -> Result<i32, FoaError<CTX>> {
        let app_cfg_info = CTX::cfg();
        let cfg = app_cfg_info.ref_into();

        let age: i32 = sqlx::query_scalar("select age from users where name=$1;")
            .bind(cfg.name)
            .fetch_one(&mut **tx)
            .await?;

        Ok(age)
    }
}

//=================
// This section depends on dependencies implementations

// *** N/A ***

//=================
// This section depends on application configuration implementation

impl<'a> RefInto<'a, ReadDafCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> ReadDafCfgInfo {
        ReadDafCfgInfo { name: &self.x }
    }
}

//=================
// This section has additional platform technology-specific code

// *** N/A ***
