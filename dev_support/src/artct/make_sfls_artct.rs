use super::{
    common::{AppCfgInfo, AppErr, DbClientDefault, DbCtx, get_app_configuration},
    FooArtctIn, FooArtctOut, FooArtctSflI,
};
use foa::context::{Cfg, CfgCtx};
use crate::artct::common::AsyncFnTx;

struct Ctx;

struct CtxCfg;

impl Cfg for CtxCfg {
    type Info = AppCfgInfo;

    fn cfg() -> Self::Info {
        get_app_configuration()
    }
}

impl CfgCtx for Ctx {
    type Cfg = CtxCfg;
}

struct CtxDbClient;

impl DbClientDefault for CtxDbClient {}

impl DbCtx for Ctx {
    type DbClient = CtxDbClient;
}

pub async fn foo_artct_sfl(input: FooArtctIn) -> Result<FooArtctOut, AppErr> {
    FooArtctSflI::<Ctx>::exec_with_transaction(input).await
}
