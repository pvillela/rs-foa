use std::fmt::Debug;

use dev_support::artctps::{
    common::AppErr, BarBfCfgInfo, FooCtx, FooIn, FooOut, FooSflCfgInfo, FooSflI,
};
use foa::{
    context::{Cfg, CfgCtx},
    db::sqlx::pg::{AsyncFnTx, Db, Itself},
    error::FoaError,
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

async fn foo_sfl<CTX>(input: FooIn) -> Result<FooOut, FoaError<CTX>>
where
    CTX: FooCtx + Db + Itself<CTX>,
{
    FooSflI::<CTX>::exec_with_transaction(input).await
}

pub async fn common_test<CTX>() -> Option<String>
where
    CTX: CfgCtx<Cfg: Cfg<Info = CfgTestInput>> + Db + Itself<CTX> + 'static + Send + Debug,
{
    let handle = tokio::spawn(async move { foo_sfl::<CTX>(FooIn { sleep_millis: 0 }).await });
    let res = handle.await.ok().map(|x| format!("{:?}", x));
    println!("{:?}", res);
    res
}
