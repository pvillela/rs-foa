use std::fmt::Debug;

use dev_support::artctps::{
    BarBfCfgInfo, FooIn, FooSflCfgInfo, FooSflI, ReadDafCfgInfo, UpdateDafCfgInfo,
};
use foa::{
    context::{Cfg, DbCtx},
    db::sqlx::pg::Db,
    refinto::RefInto,
};
use tokio;

pub struct BarBfCfgTestInput {
    pub u: i32,
}

pub struct FooSflCfgTestInput {
    pub a: String,
}

pub struct CfgTestInput {
    pub bar: BarBfCfgTestInput,
    pub foo: FooSflCfgTestInput,
}

impl<'a> RefInto<'a, BarBfCfgInfo> for CfgTestInput {
    fn ref_into(&'a self) -> BarBfCfgInfo {
        BarBfCfgInfo {
            age_increment: self.bar.u,
        }
    }
}

impl<'a> RefInto<'a, ReadDafCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> ReadDafCfgInfo<'a> {
        ReadDafCfgInfo { name: &self.foo.a }
    }
}

impl<'a> RefInto<'a, UpdateDafCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> UpdateDafCfgInfo<'a> {
        UpdateDafCfgInfo { name: &self.foo.a }
    }
}

impl<'a> RefInto<'a, FooSflCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> FooSflCfgInfo<'a> {
        FooSflCfgInfo { name: &self.foo.a }
    }
}

pub async fn common_test<CTX>() -> Option<String>
where
    CTX: Cfg<CfgInfo = CfgTestInput> + DbCtx<Db: Db> + 'static + Send + Debug,
{
    let handle = tokio::spawn(async move { FooSflI::<CTX>::sfl(FooIn { age_delta: 0 }).await });
    let res = handle.await.ok().map(|x| format!("{:?}", x));
    println!("{:?}", res);
    res
}
