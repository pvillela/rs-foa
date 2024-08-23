use super::{common::AppCfgInfoArc, BarBf, BarBfBoot, BarCtx};
use axum;
use axum::response::{IntoResponse, Response};
use foa::db::sqlx::pg::AsyncFnTx;
use foa::{
    context::{Cfg, CfgCtx},
    db::sqlx::pg::Db,
    error::FoaError,
    refinto::RefInto,
};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use std::marker::PhantomData;
use std::time::Duration;
use tokio::time::sleep;
use tracing::instrument;

#[derive(Clone, Deserialize, Debug)]
pub struct FooIn {
    pub sleep_millis: u64,
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
    pub a: &'a str,
    pub b: i32,
}

impl<'a> RefInto<'a, FooSflCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> FooSflCfgInfo<'a> {
        FooSflCfgInfo {
            a: &self.x,
            b: self.y,
        }
    }
}

pub trait FooSfl<CTX> {
    #[allow(async_fn_in_trait)]
    async fn foo_sfl(input: FooIn, tx: &mut PgConnection) -> Result<FooOut, FoaError<CTX>>;
}

pub trait FooOnlyCtx: CfgCtx<Cfg: Cfg<Info: for<'a> RefInto<'a, FooSflCfgInfo<'a>>>> {}

impl<CTX> FooOnlyCtx for CTX
where
    CTX: CfgCtx,
    <CTX::Cfg as Cfg>::Info: for<'a> RefInto<'a, FooSflCfgInfo<'a>>,
{
}

fn foo_core(a: String, b: i32, bar_res: String) -> String {
    let a = a + "-foo";
    let b = b + 3;
    format!("foo: a={}, b={}, bar=({})", a, b, bar_res)
}

pub trait FooSflC<CTX>: BarBf<CTX>
where
    CTX: FooOnlyCtx,
{
    #[instrument(level = "trace", skip_all)]
    #[allow(async_fn_in_trait)]
    async fn foo_sfl_c(input: FooIn, tx: &mut PgConnection) -> Result<FooOut, FoaError<CTX>> {
        let app_cfg_info = CTX::Cfg::cfg();
        let cfg = app_cfg_info.ref_into();
        let FooIn { sleep_millis } = input;
        sleep(Duration::from_millis(sleep_millis)).await;
        let a = cfg.a.to_owned();
        let b = cfg.b;
        let bar_res = Self::bar_bf(0, tx).await?;
        let res = foo_core(a, b, bar_res);
        Ok(FooOut { res })
    }
}

//==================
// Addition of type dependencies

pub trait FooCtx: FooOnlyCtx + BarCtx {}

impl<CTX> FooCtx for CTX where CTX: FooOnlyCtx + BarCtx {}

pub struct FooSflI<CTX>(PhantomData<CTX>);

impl<CTX> BarBfBoot<CTX> for FooSflI<CTX> where CTX: BarCtx {}
impl<CTX> FooSflC<CTX> for FooSflI<CTX> where CTX: FooCtx {}
impl<CTX> FooSflBoot<CTX> for FooSflI<CTX> where CTX: FooCtx {}

pub trait FooSflBoot<CTX>
where
    CTX: FooCtx,
{
    #[allow(async_fn_in_trait)]
    async fn foo_sfl_boot(input: FooIn, tx: &mut PgConnection) -> Result<FooOut, FoaError<CTX>> {
        FooSflI::<CTX>::foo_sfl_c(input, tx).await
    }
}

impl<CTX, T> FooSfl<CTX> for T
where
    T: FooSflBoot<CTX>,
    CTX: FooCtx,
{
    async fn foo_sfl(input: FooIn, tx: &mut PgConnection) -> Result<FooOut, FoaError<CTX>> {
        T::foo_sfl_boot(input, tx).await
    }
}

impl<CTX> AsyncFnTx<CTX, FooIn, FooOut> for FooSflI<CTX>
where
    CTX: Db + FooCtx,
{
    async fn f(input: FooIn, tx: &mut PgConnection) -> Result<FooOut, FoaError<CTX>> {
        FooSflI::foo_sfl(input, tx).await
    }
}
