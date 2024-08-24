use super::common::AppCfgInfoArc;
use foa::{
    context::{Cfg, CfgCtx},
    db::sqlx::pg::{AsyncFnTx, Db, Itself},
    error::FoaError,
    refinto::RefInto,
};
use sqlx::PgConnection;
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
    async fn bar_bf(sleep_millis: u64, tx: &mut PgConnection) -> Result<String, FoaError<CTX>>;
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
    async fn bar_bf_boot(
        sleep_millis: u64,
        _conn: &mut PgConnection,
    ) -> Result<String, FoaError<CTX>> {
        let app_cfg_info = CTX::Cfg::cfg();
        let cfg = app_cfg_info.ref_into();
        sleep(Duration::from_millis(sleep_millis)).await;
        let u = cfg.u;
        let v = cfg.v.to_owned();
        let res = bar_core(u, v);
        Ok(res)
    }
}

impl<CTX, T> BarBf<CTX> for T
where
    T: BarBfBoot<CTX>,
    CTX: BarCtx,
{
    async fn bar_bf(sleep_millis: u64, conn: &mut PgConnection) -> Result<String, FoaError<CTX>> {
        Self::bar_bf_boot(sleep_millis, conn).await
    }
}

pub struct BarBfBootI<CTX>(PhantomData<CTX>);

impl<CTX> BarBfBoot<CTX> for BarBfBootI<CTX> where CTX: BarCtx {}

impl<CTX> AsyncFnTx<CTX, BarIn, BarOut> for BarBfBootI<CTX>
where
    CTX: BarCtx + Db + Itself<CTX>,
{
    async fn f(input: BarIn, conn: &mut PgConnection) -> Result<BarOut, FoaError<CTX>> {
        BarBfBootI::<CTX>::bar_bf(input, conn).await
    }
}
