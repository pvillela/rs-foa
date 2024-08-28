use super::common::{AppCfgInfoArc, AppErr, DbCtx, DummyTx};
use crate::artct::common::AsyncFnTx;
use foa::{
    context::{Cfg, CfgCtx},
    refinto::RefInto,
};
use std::marker::PhantomData;
use std::time::Duration;
use tokio::time::sleep;
use tracing::instrument;

pub type BarArtctIn = u64;
pub type BarArtctOut = String;

pub struct BarArtctBfCfgInfo<'a> {
    pub u: i32,
    pub v: &'a str,
}

impl<'a> RefInto<'a, BarArtctBfCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> BarArtctBfCfgInfo<'a> {
        BarArtctBfCfgInfo {
            u: self.y,
            v: &self.x,
        }
    }
}

pub trait BarArtctBf<CTX> {
    #[allow(async_fn_in_trait)]
    async fn bar_artct_bf(sleep_millis: u64, tx: &DummyTx<'_>) -> Result<String, AppErr>;
}

pub trait BarCtx: CfgCtx<Cfg: Cfg<CfgInfo: for<'a> RefInto<'a, BarArtctBfCfgInfo<'a>>>> {}

impl<CTX> BarCtx for CTX
where
    CTX: CfgCtx,
    <CTX::Cfg as Cfg>::CfgInfo: for<'a> RefInto<'a, BarArtctBfCfgInfo<'a>>,
{
}

pub fn bar_core(u: i32, v: String) -> String {
    let u = u + 1;
    let v = v + "-bar";
    format!("bar: u={}, v={}", u, v)
}

pub trait BarArtctBfBoot<CTX>
where
    CTX: BarCtx,
{
    #[instrument(level = "trace", skip_all)]
    #[allow(async_fn_in_trait)]
    async fn bar_artct_bf_boot(sleep_millis: u64, tx: &DummyTx<'_>) -> Result<String, AppErr> {
        let app_cfg_info = CTX::Cfg::cfg();
        let cfg = app_cfg_info.ref_into();
        sleep(Duration::from_millis(sleep_millis)).await;
        let u = cfg.u;
        let v = cfg.v.to_owned();
        let res = bar_core(u, v) + &tx.dummy("bar_artct_bf_c");
        Ok(res)
    }
}

impl<CTX, T> BarArtctBf<CTX> for T
where
    T: BarArtctBfBoot<CTX>,
    CTX: BarCtx,
{
    async fn bar_artct_bf(sleep_millis: u64, tx: &DummyTx<'_>) -> Result<String, AppErr> {
        Self::bar_artct_bf_boot(sleep_millis, tx).await
    }
}

pub struct BarArtctBfBootI<CTX>(PhantomData<CTX>);

impl<CTX> BarArtctBfBoot<CTX> for BarArtctBfBootI<CTX> where CTX: BarCtx {}

impl<CTX, T> AsyncFnTx<CTX, BarArtctIn, BarArtctOut> for T
where
    CTX: DbCtx,
    T: BarArtctBf<CTX>,
{
    async fn f(input: BarArtctIn, tx: &DummyTx<'_>) -> Result<BarArtctOut, AppErr> {
        T::bar_artct_bf(input, tx).await
    }
}
