use super::common::AppCfgInfoArc;
use foa::{context::Cfg, refinto::RefInto};
use tracing::instrument;

//=================
// This section defines the stereotype signature

pub trait BarBf<CTX> {
    fn bar_bf(base_age: i32, age_delta: i32) -> i32;
}

//=================
// This section implements the stereotype but depends on signatures only

pub struct BarBfCfgInfo {
    pub age_increment: i32,
}

/// Trait alias
pub trait BarCtx: Cfg<CfgInfo: for<'a> RefInto<'a, BarBfCfgInfo>> {}
impl<CTX> BarCtx for CTX
where
    CTX: Cfg,
    CTX::CfgInfo: for<'a> RefInto<'a, BarBfCfgInfo>,
{
}

impl<CTX, T> BarBf<CTX> for T
where
    CTX: BarCtx,
{
    #[instrument(level = "trace", skip_all)]
    fn bar_bf(base_age: i32, age_delta: i32) -> i32 {
        let app_cfg_info = CTX::cfg();
        let cfg = app_cfg_info.ref_into();
        base_age + age_delta + cfg.age_increment
    }
}

//=================
// This section depends on dependencies implementations

// *** N/A ***

//=================
// This section depends on application configuration implementation

impl<'a> RefInto<'a, BarBfCfgInfo> for AppCfgInfoArc {
    fn ref_into(&'a self) -> BarBfCfgInfo {
        BarBfCfgInfo {
            age_increment: self.y,
        }
    }
}

//=================
// This section has additional platform technology-specific code

// *** N/A ***
