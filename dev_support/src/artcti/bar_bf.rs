use super::common::{AppCfgInfoArc, AppErr, DbCtx, DummyTx, Transaction};
use crate::artcti::common::AsyncFnTx;
use foa::{
    context::{Cfg, CfgCtx},
    refinto::RefInto,
};
use std::marker::PhantomData;
use std::time::Duration;
use tokio::time::sleep;
use tracing::instrument;

pub type BarIn = u64;
pub type BarOut = String;

pub struct BarBfCfgInfo<'a> {
    pub u: i32,
    pub v: &'a str,
}

impl<'a> RefInto<'a, BarBfCfgInfo<'a>> for AppCfgInfoArc {
    fn ref_into(&'a self) -> BarBfCfgInfo<'a> {
        BarBfCfgInfo {
            u: self.y,
            v: &self.x,
        }
    }
}

pub trait BarBf<CTX> {
    #[allow(async_fn_in_trait)]
    async fn bar_bf(sleep_millis: u64, tx: &DummyTx<'_>) -> Result<String, AppErr>;
}

pub trait BarCtx: CfgCtx<Cfg: Cfg<Info: for<'a> RefInto<'a, BarBfCfgInfo<'a>>>> {}

impl<CTX> BarCtx for CTX
where
    CTX: CfgCtx,
    <CTX::Cfg as Cfg>::Info: for<'a> RefInto<'a, BarBfCfgInfo<'a>>,
{
}

pub fn bar_core(u: i32, v: String) -> String {
    let u = u + 1;
    let v = v + "-bar";
    format!("bar: u={}, v={}", u, v)
}

pub trait BarBfBoot<CTX>
where
    CTX: BarCtx,
{
    #[instrument(level = "trace", skip_all)]
    #[allow(async_fn_in_trait)]
    async fn bar_bf_boot(sleep_millis: u64, tx: &DummyTx<'_>) -> Result<String, AppErr> {
        let app_cfg_info = CTX::Cfg::cfg();
        let cfg = app_cfg_info.ref_into();
        sleep(Duration::from_millis(sleep_millis)).await;
        let u = cfg.u;
        let v = cfg.v.to_owned();
        let res = bar_core(u, v) + &tx.dummy("bar_bf_c");
        Ok(res)
    }
}

impl<CTX, T> BarBf<CTX> for T
where
    T: BarBfBoot<CTX>,
    CTX: BarCtx,
{
    async fn bar_bf(sleep_millis: u64, tx: &DummyTx<'_>) -> Result<String, AppErr> {
        Self::bar_bf_boot(sleep_millis, tx).await
    }
}

pub struct BarBfBootI<CTX>(PhantomData<CTX>);

impl<CTX> BarBfBoot<CTX> for BarBfBootI<CTX> where CTX: BarCtx {}

impl<CTX, T> AsyncFnTx<CTX, BarIn, BarOut> for T
where
    CTX: Transaction,
    T: BarBf<CTX>,
{
    async fn f(input: BarIn, tx: &DummyTx<'_>) -> Result<BarOut, AppErr> {
        T::bar_bf(input, tx).await
    }
}
