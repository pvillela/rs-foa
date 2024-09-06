use super::{common::AppCfgInfoArc, BarBf, BarCtx, ReadDaf, ReadDafCtx, UpdateDaf, UpdateDafCtx};
use foa::{
    context::{Cfg, Locale, LocaleCtx},
    db::sqlx::{AsyncTxFn, PgDbCtx},
    error::FoaError,
    refinto::RefInto,
    trait_utils::Make,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use std::marker::PhantomData;
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
    CTX: FooOnlyCtx + LocaleCtx,
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
        Self::update_daf(new_age, tx).await?;
        Ok(FooOut {
            name: cfg.name.into(),
            new_age,
            refresh_count: cfg.count,
            locale: locale.to_string(),
        })
    }
}

//=================
// This section depends on dependencies implementations

/// Trait alias
pub trait FooCtx: FooOnlyCtx + BarCtx + ReadDafCtx + UpdateDafCtx {}
impl<CTX> FooCtx for CTX where CTX: FooOnlyCtx + BarCtx + ReadDafCtx + UpdateDafCtx {}

/// Any type parameterized by `CTX` where `CTX: FooCtx` implements `FooSfl<CTX>` as
/// it is recursively true for its dependencies.
#[cfg(test)]
#[allow(unused)]
mod illustrative {
    use super::*;

    trait FooSflAlias<CTX>: FooSfl<CTX> {}
    impl<CTX, T> FooSflAlias<CTX> for T where CTX: FooCtx + LocaleCtx {}
}

/// Stereotype instance
#[derive(Clone)]
pub struct FooSflI<CTX: FooCtx>(PhantomData<CTX>);

impl<CTX: FooCtx> Make<Self> for FooSflI<CTX> {
    fn make() -> Self {
        FooSflI(PhantomData)
    }
}

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

impl<CTX> AsyncTxFn<CTX> for FooSflI<CTX>
where
    CTX: FooCtx + LocaleCtx + PgDbCtx + Sync,
    CTX::CfgInfo: Send,
{
    type In = FooIn;
    type Out = FooOut;
    type E = FoaError<CTX>;

    async fn invoke(
        &self,
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>> {
        FooSflI::<CTX>::foo_sfl(input, tx).await
    }
}