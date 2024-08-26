use std::fmt::Debug;

use dev_support::artctps::{BarBfCfgInfo, FooIn, FooSflCfgInfo, FooSflI};
use foa::context::Itself;
use foa::{
    context::{Cfg, CfgCtx},
    db::sqlx::pg::Db,
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

impl<'a> RefInto<'a, BarBfCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> BarBfCfgInfo<'a> {
        BarBfCfgInfo {
            u: self.bar.u,
            v: &self.bar.v,
        }
    }
}

impl<'a> RefInto<'a, FooSflCfgInfo<'a>> for CfgTestInput {
    fn ref_into(&'a self) -> FooSflCfgInfo<'a> {
        FooSflCfgInfo {
            a: &self.foo.a,
            b: self.foo.b,
        }
    }
}

pub async fn common_test<CTX>() -> Option<String>
where
    CTX: CfgCtx<Cfg: Cfg<Info = CfgTestInput>> + Db + Itself<CTX> + 'static + Send + Debug,
{
    let handle = tokio::spawn(async move { FooSflI::<CTX>::sfl(FooIn { sleep_millis: 0 }).await });
    let res = handle.await.ok().map(|x| format!("{:?}", x));
    println!("{:?}", res);
    res
}
