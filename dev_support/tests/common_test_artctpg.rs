use dev_support::artctpg::{
    BarBfCfgInfo, FooCtx, FooIn, FooOut, FooSfl, FooSflCfgInfo, FooSflI, InitDaf, InitDafCfgInfo,
    InitDafCtx, InitDafI, ReadDafCfgInfo, UpdateDafCfgInfo,
};
use foa::{
    context::Cfg,
    db::sqlx::{AsyncTxFn, PgDbCtx},
    error::FoaError,
    refinto::RefInto,
};
use sqlx::{Postgres, Transaction};
use std::{fmt::Debug, marker::PhantomData};
use tokio::{self};

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

pub struct TestFooSflI<CTX>(PhantomData<CTX>);

impl<CTX> AsyncTxFn<CTX> for TestFooSflI<CTX>
where
    CTX: FooCtx + InitDafCtx + PgDbCtx,
{
    type In = FooIn;
    type Out = FooOut;
    type E = FoaError<CTX>;

    async fn call(
        input: FooIn,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<FooOut, FoaError<CTX>> {
        InitDafI::<CTX>::init_daf(tx).await?;
        FooSflI::<CTX>::foo_sfl(input, tx).await
    }
}

pub async fn common_test<CTX>() -> Result<FooOut, FoaError<CTX>>
where
    CTX: Cfg<CfgInfo = CfgTestInput> + PgDbCtx + 'static + Send + Debug,
{
    InitDafI::<CTX>::in_tx(()).await?;
    let handle =
        tokio::spawn(async move { TestFooSflI::<CTX>::in_tx(FooIn { age_delta: 1 }).await });
    handle.await.expect("common_test_artctps tokio spawn error")
}
