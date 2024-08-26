use arc_swap::{ArcSwapAny, RefCnt};
use std::{fmt::Debug, sync::OnceLock};

//=============
// Context traits

pub trait RefCntWrapper: Sized {
    type Inner: RefCnt;

    fn wrap(inner: Self::Inner) -> Self;
    fn inner(&self) -> Self::Inner;
}

pub trait Context: RefCntWrapper + 'static {
    type CfgInfo;

    fn ctx_static() -> &'static OnceLock<ArcSwapAny<Self::Inner>>;
    fn new_inner() -> Self::Inner;
    fn inner_with_updated_app_cfg(inner: &Self::Inner, cfg_info: Self::CfgInfo) -> Self::Inner;
    fn get_app_cfg(&self) -> Self::CfgInfo;

    fn refresh_app_cfg(app_cfg: Self::CfgInfo) {
        let ctx_asw = get_ctx_arcswap::<Self>();
        let inner = ctx_asw.load();
        let inner = Self::inner_with_updated_app_cfg(&inner, app_cfg);
        ctx_asw.store(inner);
    }
}

fn get_ctx_arcswap<T: Context>() -> &'static ArcSwapAny<T::Inner> {
    T::ctx_static().get_or_init(|| ArcSwapAny::from(T::new_inner()))
}

impl<T> Itself<T> for T
where
    T: Context,
{
    fn itself() -> Self {
        let ctx_asw = get_ctx_arcswap::<Self>();
        Self::wrap(ctx_asw.load().clone())
    }
}

pub trait Cfg {
    type Info;

    fn cfg() -> Self::Info;
}

impl<T> Cfg for T
where
    T: Context,
{
    type Info = T::CfgInfo;

    fn cfg() -> Self::Info {
        Self::itself().get_app_cfg()
    }
}

pub trait CfgCtx {
    type Cfg: Cfg;
}

impl<T> CfgCtx for T
where
    T: Context,
{
    type Cfg = T;
}

pub trait LocalizedMsg {
    fn localized_msg<'a>(kind: &'a str, locale: &'a str) -> Option<&'a str>;
}

pub trait Locale {
    fn locale<'a>() -> &'a str;
}

pub trait ErrCtx: Debug + Send + Sync + 'static {
    type Locale: Locale;
    type LocalizedMsg: LocalizedMsg;
}

pub trait DbCtx {
    type Db;
}

pub trait SecCtx {
    // TBD
}

//=============
// impls for NullCtx

pub type NullCtx = ();

pub struct NullCtxTypeI;

impl LocalizedMsg for NullCtxTypeI {
    fn localized_msg<'a>(_kind: &'a str, _locale: &'a str) -> Option<&'a str> {
        None
    }
}

impl Locale for NullCtxTypeI {
    fn locale<'a>() -> &'a str {
        ""
    }
}

impl ErrCtx for NullCtx {
    type Locale = NullCtxTypeI;
    type LocalizedMsg = NullCtxTypeI;
}

pub trait Itself<CTX> {
    fn itself() -> CTX;
}
