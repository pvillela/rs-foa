use arc_swap::{ArcSwap, ArcSwapAny};
use std::sync::{Arc, OnceLock};

pub trait AppCfg: Sized + 'static {
    fn app_cfg_static() -> &'static OnceLock<ArcSwap<Self>>;
    fn app_config_info() -> Self;

    // Refreshes of APP_CONFIGURATION
    fn refresh_app_configuration() {
        let cfg_as = get_app_config_arcswap();
        cfg_as.store(Arc::new(Self::app_config_info()));
    }

    fn get_app_configuration() -> Arc<Self> {
        let cfg_as = get_app_config_arcswap::<Self>();
        cfg_as.load().clone()
    }
}

fn get_app_config_arcswap<T: AppCfg>() -> &'static ArcSwapAny<Arc<T>> {
    T::app_cfg_static().get_or_init(|| ArcSwap::from_pointee(T::app_config_info()))
}
