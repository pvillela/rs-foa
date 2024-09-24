use super::common::AppCfgInfoArc;
use foa::{context::Cfg, refinto::RefInto};
use tracing::instrument;

// region:      --- Stereotype signature

pub trait BarBf<CTX> {
    fn bar_bf(base_age: i32, age_delta: i32) -> i32;
}
// endregion:   --- Stereotype signature

// region:      --- Stereotype implementation with dependencies' signatures only

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

// endregion:   --- Stereotype implementation with dependencies' signatures only

// region:      --- Depends on dependencies' implementations

// *** N/A ***

// endregion:   --- Depends on dependencies' implementations

// region:      --- Additional platform technology-specific code

// *** N/A ***

// endregion:   --- Additional platform technology-specific code

// region:      --- Depends on application configuration implementation

impl<'a> RefInto<'a, BarBfCfgInfo> for AppCfgInfoArc {
    fn ref_into(&'a self) -> BarBfCfgInfo {
        BarBfCfgInfo {
            age_increment: self.y,
        }
    }
}
// endregion:   --- Depends on application configuration implementation
