use std::fmt::Debug;

use dev_support::artctps::{
    BarBfCfgInfo, FooIn, FooOut, FooSflCfgInfo, FooSflI, InitDafCfgInfo, InitDafI, ReadDafCfgInfo,
    UpdateDafCfgInfo,
};
use foa::{
    context::Cfg,
    db::sqlx::{AsyncTxFn, PgDb},
    error::FoaError,
    refinto::RefInto,
};
use tokio;

pub struct BarBfCfgTestInput {
    pub incr: i32,
}

pub struct FooSflCfgTestInput {
    pub n: String,
    pub c: u32,
}

pub struct InitDafCfgTestinput {
    pub init_age: i32,
}

pub struct CfgTestInput {
    pub bar: BarBfCfgTestInput,
    pub foo: FooSflCfgTestInput,
    pub init: InitDafCfgTestinput,
}

impl<'a> RefInto<'a, BarBfCfgInfo> for CfgTestInput {
    fn ref_into(&'a self) -> BarBfCfgInfo {
        BarBfCfgInfo {
            age_increment: self.bar.incr,
        }
    }
}

impl<'a> RefInto<'a, ReadDafCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> ReadDafCfgInfo<'a> {
        ReadDafCfgInfo { name: &self.foo.n }
    }
}

impl<'a> RefInto<'a, UpdateDafCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> UpdateDafCfgInfo<'a> {
        UpdateDafCfgInfo { name: &self.foo.n }
    }
}

impl<'a> RefInto<'a, FooSflCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> FooSflCfgInfo<'a> {
        FooSflCfgInfo {
            name: &self.foo.n,
            count: self.foo.c,
        }
    }
}

impl<'a> RefInto<'a, InitDafCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> InitDafCfgInfo<'a> {
        InitDafCfgInfo {
            name: &self.foo.n,
            initial_age: self.init.init_age,
        }
    }
}

pub async fn common_test<CTX>() -> Result<FooOut, FoaError<CTX>>
where
    CTX: Cfg<CfgInfo = CfgTestInput> + PgDb + 'static + Send + Debug,
{
    InitDafI::<CTX>::in_tx(()).await?;
    let handle = tokio::spawn(async move { FooSflI::<CTX>::in_tx(FooIn { age_delta: 1 }).await });
    handle.await.expect("common_test_artctps tokio spawn error")
}
