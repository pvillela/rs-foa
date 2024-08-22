use super::{
    common::{AppCfgInfo, AppCfgInfoArc, AppErr, DbClientDefault, DbCtx},
    FooArtctIn, FooArtctOut, FooArtctSflI,
};
use crate::artct::common::AsyncFnTx;
use foa::{
    appcfg::AppCfg,
    context::{Cfg, CfgCtx},
};

struct Ctx;

struct CtxCfg;

impl Cfg for CtxCfg {
    type Info = AppCfgInfoArc;

    fn cfg() -> Self::Info {
        AppCfgInfo::get_app_configuration()
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
