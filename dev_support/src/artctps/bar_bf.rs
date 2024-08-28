use super::common::AppCfgInfoArc;
use foa::{context::Cfg, refinto::RefInto};
use tracing::instrument;

pub struct BarBfCfgInfo {
    pub age_increment: i32,
}

impl<'a> RefInto<'a, BarBfCfgInfo> for AppCfgInfoArc {
    fn ref_into(&'a self) -> BarBfCfgInfo {
        BarBfCfgInfo {
            age_increment: self.y,
        }
    }
}

pub trait BarBf<CTX> {
    fn bar_bf(base_age: i32, age_delta: i32) -> i32;
}

/// Trait alias
pub trait BarCtx: Cfg<CfgInfo: for<'a> RefInto<'a, BarBfCfgInfo>> {}
impl<CTX> BarCtx for CTX
where
    CTX: Cfg,
    CTX::CfgInfo: for<'a> RefInto<'a, BarBfCfgInfo>,
{
}

pub trait BarBfBoot<CTX>
where
    CTX: BarCtx,
{
    #[instrument(level = "trace", skip_all)]
    fn bar_bf_boot(base_age: i32, age_delta: i32) -> i32 {
        let app_cfg_info = CTX::cfg();
        let cfg = app_cfg_info.ref_into();
        base_age + age_delta + cfg.age_increment
    }
}

impl<CTX, T> BarBf<CTX> for T
where
    T: BarBfBoot<CTX>,
    CTX: BarCtx,
{
    fn bar_bf(base_age: i32, age_delta: i32) -> i32 {
        Self::bar_bf_boot(base_age, age_delta)
    }
}
