#![allow(deprecated)]

use dev_support::artct::{
    common::{AppErr, AsyncFnTx, DbCtx},
    BarArtctBfCfgInfo, FooArtctIn, FooArtctOut, FooArtctSflCfgInfo, FooArtctSflI, FooCtx,
};
use foa::{
    context::{Cfg, CfgCtx},
    refinto::RefInto,
};
use tokio;

pub struct BarBfCfgTestInput {
    pub u: i32,
    pub v: String,
}

pub struct FooSflCfgTestInput {
    pub a: String,
    pub b: i32,
}

pub struct CfgTestInput {
    pub bar: BarBfCfgTestInput,
    pub foo: FooSflCfgTestInput,
}

impl<'a> RefInto<'a, BarArtctBfCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> BarArtctBfCfgInfo<'a> {
        BarArtctBfCfgInfo {
            u: self.bar.u,
            v: &self.bar.v,
        }
    }
}

impl<'a> RefInto<'a, FooArtctSflCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> FooArtctSflCfgInfo<'a> {
        FooArtctSflCfgInfo {
            a: &self.foo.a,
            b: self.foo.b,
        }
    }
}

async fn foo_artct_sfl<CTX>(input: FooArtctIn) -> Result<FooArtctOut, AppErr>
where
    CTX: FooCtx + DbCtx,
{
    FooArtctSflI::<CTX>::exec_with_transaction(input).await
}

pub async fn common_test<CTX>() -> Option<String>
where
    CTX: CfgCtx<Cfg: Cfg<CfgInfo = CfgTestInput>> + DbCtx + 'static,
{
    let handle =
        tokio::spawn(async move { foo_artct_sfl::<CTX>(FooArtctIn { sleep_millis: 0 }).await });
    let res = handle.await.ok().map(|x| format!("{:?}", x));
    println!("{:?}", res);
    res
}
