use super::{common::AppCfgInfoArc, BarBf, BarCtx, ReadDaf, ReadDafCtx, UpdateDaf, UpdateDafCtx};
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
#[derive(Serialize, Debug, PartialEq)]
pub struct FooOut {
    pub name: String,
    pub new_age: i32,
    pub refresh_count: u32,
}

impl IntoResponse for FooOut {
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}

pub struct FooSflCfgInfo<'a> {
    pub name: &'a str,
    pub count: u32,
}

impl<'a> RefInto<'a, FooSflCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> FooSflCfgInfo<'a> {
        FooSflCfgInfo {
            name: &self.x,
            count: self.refresh_count,
        }
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

impl<CTX, T> FooSfl<CTX> for T
where
    CTX: FooOnlyCtx,
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
        Self::update_daf(new_age, tx).await?;
        Ok(FooOut {
            name: cfg.name.into(),
            new_age,
            refresh_count: cfg.count,
        })
    }
}

//==================
// Addition of type dependencies

/// Trait alias
pub trait FooCtx: FooOnlyCtx + BarCtx + ReadDafCtx + UpdateDafCtx {}
impl<CTX> FooCtx for CTX where CTX: FooOnlyCtx + BarCtx + ReadDafCtx + UpdateDafCtx {}

/// Stereotype instance
pub struct FooSflI<CTX>(PhantomData<CTX>);

impl<CTX> PgSfl<FooIn, Result<FooOut, FoaError<CTX>>> for FooSflI<CTX>
where
    CTX: FooCtx,
{
    async fn sfl(
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>> {
        FooSflI::<CTX>::foo_sfl(input, tx).await
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
