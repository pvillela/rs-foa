use super::{common::AppCfgInfoArc, BarBf, BarCtx, ReadDaf, ReadDafCtx, UpdateDaf, UpdateDafCtx};
use axum::http::request::Parts;
use foa::{
    context::{Cfg, Locale, LocaleCtx},
    db::sqlx::{AsyncTxFn, PgDbCtx},
    error::FoaError,
    refinto::RefInto,
    tokio::task_local::{TaskLocal, TaskLocalCtx},
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use tracing::instrument;
//=================
// This section defines the stereotype signature

#[derive(Clone, Deserialize, Debug)]
pub struct FooIn {
    pub age_delta: i32,
}

#[allow(unused)]
#[derive(Serialize, Debug, PartialEq)]
pub struct FooOut {
    pub name: String,
    pub new_age: i32,
    pub refresh_count: u32,
    pub locale: String,
    pub method: String,
    pub uri: String,
    pub version: String,
    pub extensions: String,
}

pub trait FooSfl<CTX> {
    #[allow(async_fn_in_trait)]
    async fn foo_sfl(
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>>;
}

//=================
// This section implements the stereotype but depends on signatures only

pub struct FooSflCfgInfo<'a> {
    pub name: &'a str,
    pub count: u32,
}

/// Trait alias
pub trait FooOnlyCtx: Cfg<CfgInfo: for<'a> RefInto<'a, FooSflCfgInfo<'a>>> {}
impl<CTX> FooOnlyCtx for CTX
where
    CTX: Cfg,
    CTX::CfgInfo: for<'a> RefInto<'a, FooSflCfgInfo<'a>>,
{
}

impl<CTX, T> FooSfl<CTX> for T
where
    CTX: FooOnlyCtx + LocaleCtx + TaskLocalCtx<TaskLocal: TaskLocal<Value = Parts>>,
    T: BarBf<CTX> + ReadDaf<CTX> + UpdateDaf<CTX>,
{
    #[instrument(level = "trace", skip_all)]
    #[allow(async_fn_in_trait)]
    async fn foo_sfl(
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>> {
        let app_cfg_info = CTX::cfg();
        let cfg = app_cfg_info.ref_into();
        let FooIn { age_delta } = input;
        let stored_age = Self::read_daf(tx).await?;
        let new_age = Self::bar_bf(stored_age, age_delta);
        let locale = CTX::Locale::locale();
        let parts = CTX::TaskLocal::cloned_value();
        Self::update_daf(new_age, tx).await?;
        Ok(FooOut {
            name: cfg.name.into(),
            new_age,
            refresh_count: cfg.count,
            locale: locale.to_string(),
            method: parts.method.to_string(),
            uri: parts.uri.to_string(),
            version: format!("{:?}", parts.version),
            extensions: format!("{:?}", parts.extensions),
        })
    }
}

//=================
// This section depends on dependencies implementations

/// Trait alias
pub trait FooCtx:
    FooOnlyCtx
    + LocaleCtx
    + TaskLocalCtx<TaskLocal: TaskLocal<Value = Parts>>
    + BarCtx
    + ReadDafCtx
    + UpdateDafCtx
{
}
impl<CTX> FooCtx for CTX where
    CTX: FooOnlyCtx
        + FooOnlyCtx
        + LocaleCtx
        + TaskLocalCtx<TaskLocal: TaskLocal<Value = Parts>>
        + BarCtx
        + ReadDafCtx
        + UpdateDafCtx
{
}

/// Any type parameterized by `CTX` where `CTX: FooCtx` implements `FooSfl<CTX>` as
/// it is recursively true for its dependencies.
#[cfg(test)]
#[allow(unused)]
mod illustrative {
    use super::*;

    trait FooSflAlias<CTX>: FooSfl<CTX> {}
    impl<CTX, T> FooSflAlias<CTX> for T where CTX: FooCtx {}
}

/// Stereotype instance
#[derive(Clone)]
pub struct FooSflI<CTX: FooCtx>(pub CTX);

//=================
// This section depends on application configuration implementation

impl<'a> RefInto<'a, FooSflCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> FooSflCfgInfo<'a> {
        FooSflCfgInfo {
            name: &self.x,
            count: self.refresh_count,
        }
    }
}

//=================
// This section has additional platform technology-specific code

impl<CTX> AsyncTxFn for FooSflI<CTX>
where
    CTX: FooCtx + PgDbCtx + Sync,
    CTX::CfgInfo: Send,
{
    type In = FooIn;
    type Out = FooOut;
    type E = FoaError<CTX>;
    type Db = CTX::Db;

    async fn invoke(
        &self,
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>> {
        FooSflI::<CTX>::foo_sfl(input, tx).await
    }
}
