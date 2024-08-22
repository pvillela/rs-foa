use super::{
    common::{foo_core, AppCfgInfo, AppErr, DbCtx, DummyTx, FooArtIn, FooArtOut},
    BarArtctBf, BarArtctBfBoot, BarCtx,
};
use crate::artct::common::AsyncFnTx;
use foa::{
    context::{Cfg, CfgCtx},
    refinto::RefInto,
};
use std::marker::PhantomData;
use std::time::Duration;
use tokio::time::sleep;
use tracing::instrument;

pub type FooArtctIn = FooArtIn;
pub type FooArtctOut = FooArtOut;

pub struct FooArtctSflCfgInfo<'a> {
    pub a: &'a str,
    pub b: i32,
}

impl<'a> RefInto<'a, FooArtctSflCfgInfo<'a>> for AppCfgInfo {
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

pub trait FooOnlyCtx: CfgCtx<Cfg: Cfg<Info: for<'a> RefInto<'a, FooArtctSflCfgInfo<'a>>>> {}

impl<CTX> FooOnlyCtx for CTX
where
    CTX: CfgCtx,
    <CTX::Cfg as Cfg>::Info: for<'a> RefInto<'a, FooArtctSflCfgInfo<'a>>,
{
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
