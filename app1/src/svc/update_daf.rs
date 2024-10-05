use crate::svc::common::AppCfgInfoArc;
use foa::{context::Cfg, refinto::RefInto, Result};
use sqlx::{Postgres, Transaction};
use tracing::instrument;

// region:      --- Stereotype signature

pub trait UpdateDaf<CTX> {
    #[allow(async_fn_in_trait)]
    async fn update_daf(age: i32, tx: &mut Transaction<'_, Postgres>) -> Result<()>;
}

// endregion:   --- Stereotype signature

// region:      --- Stereotype implementation with dependencies' signatures only

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
    CTX: UpdateDafCtx,
{
    #[instrument(level = "trace", skip_all)]
    #[allow(async_fn_in_trait)]
    async fn update_daf(age: i32, tx: &mut Transaction<'_, Postgres>) -> Result<()> {
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

// endregion:   --- Stereotype implementation with dependencies' signatures only

// region:      --- Depends on dependencies' implementations

// *** N/A ***

// endregion:   --- Depends on dependencies' implementations

// region:      --- Additional platform technology-specific code

// *** N/A ***

// endregion:   --- Additional platform technology-specific code

// region:      --- Depends on application configuration implementation

impl<'a> RefInto<'a, UpdateDafCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> UpdateDafCfgInfo {
        UpdateDafCfgInfo { name: &self.x }
    }
}

// endregion:   --- Depends on application configuration implementation
