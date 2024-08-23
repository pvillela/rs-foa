use super::{
    common::{
        AppCfgInfo, AppCfgInfoArc, AppErr, DbClientDefault, DbCtx, DbErr, DummyTx, Transaction,
    },
    FooIn, FooOut, FooSflI,
};
use crate::artcti::common::AsyncFnTx;
use foa::{
    appcfg::AppCfg,
    context::{Cfg, CfgCtx},
};

pub struct Ctx;

pub struct CtxCfg;

impl Cfg for CtxCfg {
    type Info = AppCfgInfoArc;

    fn cfg() -> Self::Info {
        AppCfgInfo::get_app_configuration()
    }
}

impl CfgCtx for Ctx {
    type Cfg = CtxCfg;
}

// struct CtxDbClient;

// impl DbClientDefault for CtxDbClient {}

// impl DbCtx for Ctx {
//     type DbClient = CtxDbClient;
// }

impl Transaction for Ctx {
    type Tx<'a> = DummyTx<'a>;
    type DbErr = DbErr;

    async fn transaction<'a>(&'a self) -> Result<DummyTx<'a>, DbErr> {
        // TODO: implement this properly
        // println!("Db.transaction() called");
        todo!()
    }
}

pub async fn foo_sfl(ctx: &Ctx, input: FooIn) -> Result<FooOut, AppErr> {
    FooSflI::<Ctx>::exec_with_transaction(ctx, input).await
}
