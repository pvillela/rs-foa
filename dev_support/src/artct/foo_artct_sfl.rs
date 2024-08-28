use super::{
    common::{AppCfgInfoArc, AppErr, DbCtx, DummyTx},
    BarArtctBf, BarArtctBfBoot, BarCtx,
};
use crate::artct::common::AsyncFnTx;
use axum;
use axum::response::{IntoResponse, Response};
use foa::{
    context::{Cfg, CfgCtx},
    refinto::RefInto,
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::time::Duration;
use tokio::time::sleep;
use tracing::instrument;

#[derive(Clone, Deserialize, Debug)]
pub struct FooArtctIn {
    pub sleep_millis: u64,
}

#[allow(unused)]
#[derive(Serialize, Debug)]
pub struct FooArtctOut {
    pub res: String,
}

impl IntoResponse for FooArtctOut {
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}

pub struct FooArtctSflCfgInfo<'a> {
    pub a: &'a str,
    pub b: i32,
}

impl<'a> RefInto<'a, FooArtctSflCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> FooArtctSflCfgInfo<'a> {
        FooArtctSflCfgInfo {
            a: &self.x,
            b: self.y,
        }
    }
}

pub trait FooArtctSfl<CTX> {
    #[allow(async_fn_in_trait)]
    async fn foo_artct_sfl(input: FooArtctIn, tx: &DummyTx<'_>) -> Result<FooArtctOut, AppErr>;
}

pub trait FooOnlyCtx:
    CfgCtx<Cfg: Cfg<CfgInfo: for<'a> RefInto<'a, FooArtctSflCfgInfo<'a>>>>
{
}

impl<CTX> FooOnlyCtx for CTX
where
    CTX: CfgCtx,
    <CTX::Cfg as Cfg>::CfgInfo: for<'a> RefInto<'a, FooArtctSflCfgInfo<'a>>,
{
}

fn foo_core(a: String, b: i32, bar_res: String) -> String {
    let a = a + "-foo";
    let b = b + 3;
    format!("foo: a={}, b={}, bar=({})", a, b, bar_res)
}

pub trait FooArtctSflC<CTX>: BarArtctBf<CTX>
where
    CTX: FooOnlyCtx,
{
    #[instrument(level = "trace", skip_all)]
    #[allow(async_fn_in_trait)]
    async fn foo_artct_sfl_c(input: FooArtctIn, tx: &DummyTx<'_>) -> Result<FooArtctOut, AppErr> {
        let app_cfg_info = CTX::Cfg::cfg();
        let cfg = app_cfg_info.ref_into();
        let FooArtctIn { sleep_millis } = input;
        sleep(Duration::from_millis(sleep_millis)).await;
        let a = cfg.a.to_owned();
        let b = cfg.b;
        let bar_res = (Self::bar_artct_bf)(0, tx).await.unwrap();
        let res = foo_core(a, b, bar_res) + &tx.dummy("foo_artct_sfl_c");
        Ok(FooArtctOut { res })
    }
}

//==================
// Addition of type dependencies

pub trait FooCtx: FooOnlyCtx + BarCtx {}

impl<CTX> FooCtx for CTX where CTX: FooOnlyCtx + BarCtx {}

pub struct FooArtctSflI<CTX>(PhantomData<CTX>);

impl<CTX> BarArtctBfBoot<CTX> for FooArtctSflI<CTX> where CTX: BarCtx {}
impl<CTX> FooArtctSflC<CTX> for FooArtctSflI<CTX> where CTX: FooCtx {}
impl<CTX> FooArtctSflBoot<CTX> for FooArtctSflI<CTX> where CTX: FooCtx {}

pub trait FooArtctSflBoot<CTX>
where
    CTX: FooCtx,
{
    #[allow(async_fn_in_trait)]
    async fn foo_artct_sfl_boot(
        input: FooArtctIn,
        tx: &DummyTx<'_>,
    ) -> Result<FooArtctOut, AppErr> {
        FooArtctSflI::<CTX>::foo_artct_sfl_c(input, tx).await
    }
}

impl<CTX, T> FooArtctSfl<CTX> for T
where
    T: FooArtctSflBoot<CTX>,
    CTX: FooCtx,
{
    async fn foo_artct_sfl(input: FooArtctIn, tx: &DummyTx<'_>) -> Result<FooArtctOut, AppErr> {
        T::foo_artct_sfl_boot(input, tx).await
    }
}

impl<CTX, T> AsyncFnTx<CTX, FooArtctIn, FooArtctOut> for T
where
    CTX: DbCtx,
    T: FooArtctSfl<CTX>,
{
    async fn f(input: FooArtctIn, tx: &DummyTx<'_>) -> Result<FooArtctOut, AppErr> {
        T::foo_artct_sfl(input, tx).await
    }
}
