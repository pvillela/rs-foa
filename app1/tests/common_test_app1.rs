use app1::svc::{
    BarBfCfgInfo, FooCtx, FooIn, FooOut, FooSfl, FooSflCfgInfo, FooSflI, InitDaf, InitDafCfgInfo,
    InitDafCtx, InitDafI, ReadDafCfgInfo, UpdateDafCfgInfo,
};
use axum::http::request::Parts;
use foa::{
    context::{Cfg, LocaleCtx},
    db::sqlx::{AsyncTxFn, PgDbCtx},
    refinto::RefInto,
    tokio::task_local::{invoke_tl_scoped, TaskLocal, TaskLocalCtx},
    Error, Result,
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

struct TestFooSflI<CTX>(pub PhantomData<CTX>);

impl<CTX> AsyncTxFn for TestFooSflI<CTX>
where
    CTX: FooCtx + LocaleCtx + InitDafCtx + PgDbCtx + Sync + Send,
    CTX::CfgInfo: Send,
{
    type In = FooIn;
    type Out = FooOut;
    type E = Error;
    type Db = CTX::Db;

    async fn invoke(&self, input: FooIn, tx: &mut Transaction<'_, Postgres>) -> Result<FooOut> {
        <InitDafI<CTX> as InitDaf<CTX>>::init_daf(tx).await?;
        <FooSflI<CTX> as FooSfl<CTX>>::foo_sfl(input, tx).await
    }
}

pub async fn common_test<CTX>(parts: Parts) -> Result<FooOut>
where
    CTX: Cfg<CfgInfo = CfgTestInput>
        + LocaleCtx
        + TaskLocalCtx<TaskLocal: TaskLocal<Value = Parts> + Sync + Send>
        + PgDbCtx
        + 'static
        + Send
        + Sync
        + Debug,
{
    let handle = tokio::spawn(async move {
        invoke_tl_scoped::<_, <CTX as TaskLocalCtx>::TaskLocal>(
            &TestFooSflI(PhantomData::<CTX>).in_tx(),
            parts.clone(),
            FooIn { age_delta: 1 },
        )
        .await
    });
    handle.await.expect("common_test_artctps tokio spawn error")
}
