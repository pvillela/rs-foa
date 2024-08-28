use super::{
    common::AppCfgInfoArc, BarBf, BarBfBoot, BarCtx, ReadDaf, ReadDafBoot, ReadDafCtx, UpdateDaf,
    UpdateDafBoot, UpdateDafCtx,
};
use axum::{
    self,
    response::{IntoResponse, Response},
};
use foa::{
    context::{Cfg, DbCtx},
    db::sqlx::pg::{pg_sfl, Db, PgSfl},
    error::FoaError,
    refinto::RefInto,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use std::marker::PhantomData;
use tracing::instrument;

#[derive(Clone, Deserialize, Debug)]
pub struct FooIn {
    pub age_delta: i32,
}

#[allow(unused)]
#[derive(Serialize, Debug)]
pub struct FooOut {
    pub res: String,
}

impl IntoResponse for FooOut {
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}

pub struct FooSflCfgInfo<'a> {
    pub name: &'a str,
}

impl<'a> RefInto<'a, FooSflCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> FooSflCfgInfo<'a> {
        FooSflCfgInfo { name: &self.x }
    }
}

pub trait FooSfl<CTX> {
    #[allow(async_fn_in_trait)]
    async fn foo_sfl(
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>>;
}

/// Trait alias
pub trait FooOnlyCtx: Cfg<CfgInfo: for<'a> RefInto<'a, FooSflCfgInfo<'a>>> {}
impl<CTX> FooOnlyCtx for CTX
where
    CTX: Cfg,
    CTX::CfgInfo: for<'a> RefInto<'a, FooSflCfgInfo<'a>>,
{
}

pub trait FooSflC<CTX>: BarBf<CTX> + ReadDaf<CTX> + UpdateDaf<CTX>
where
    CTX: FooOnlyCtx,
{
    #[instrument(level = "trace", skip_all)]
    #[allow(async_fn_in_trait)]
    async fn foo_sfl_c(
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>> {
        let app_cfg_info = CTX::cfg();
        let cfg = app_cfg_info.ref_into();
        let FooIn { age_delta } = input;
        let base_age = Self::read_daf(tx).await?;
        let bar_res = Self::bar_bf(base_age, age_delta);
        Self::update_daf(bar_res, tx).await?;
        let res = format!("{}'s age is updated to {}", cfg.name, bar_res);
        Ok(FooOut { res })
    }
}

//==================
// Addition of type dependencies

/// Trait alias
pub trait FooCtx: FooOnlyCtx + BarCtx + ReadDafCtx + UpdateDafCtx {}
impl<CTX> FooCtx for CTX where CTX: FooOnlyCtx + BarCtx + ReadDafCtx + UpdateDafCtx {}

pub struct FooSflI<CTX>(PhantomData<CTX>);

impl<CTX> BarBfBoot<CTX> for FooSflI<CTX> where CTX: BarCtx {}
impl<CTX> ReadDafBoot<CTX> for FooSflI<CTX> where CTX: ReadDafCtx {}
impl<CTX> UpdateDafBoot<CTX> for FooSflI<CTX> where CTX: UpdateDafCtx {}
impl<CTX> FooSflC<CTX> for FooSflI<CTX> where CTX: FooCtx {}
impl<CTX> FooSflBoot<CTX> for FooSflI<CTX> where CTX: FooCtx {}

pub trait FooSflBoot<CTX>
where
    CTX: FooCtx,
{
    #[allow(async_fn_in_trait)]
    async fn foo_sfl_boot(
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>> {
        FooSflI::<CTX>::foo_sfl_c(input, tx).await
    }
}

impl<CTX, T> FooSfl<CTX> for T
where
    T: FooSflBoot<CTX>,
    CTX: FooCtx,
{
    async fn foo_sfl(
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>> {
        T::foo_sfl_boot(input, tx).await
    }
}

impl<CTX> PgSfl<FooIn, Result<FooOut, FoaError<CTX>>> for FooSflI<CTX>
where
    CTX: FooCtx,
{
    async fn sfl(
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>> {
        FooSflI::foo_sfl(input, tx).await
    }
}

impl<CTX> FooSflI<CTX>
where
    CTX: FooCtx + DbCtx<Db: Db>,
{
    pub async fn sfl(input: FooIn) -> Result<FooOut, FoaError<CTX>> {
        pg_sfl::<CTX, FooIn, FooOut, FoaError<CTX>, FooSflI<CTX>>(input).await
    }
}
