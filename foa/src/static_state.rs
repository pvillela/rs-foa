use arc_swap::ArcSwap;
use std::{
    future::Future,
    sync::{Arc, OnceLock},
};

pub trait StaticState<S: 'static> {
    fn get_static() -> &'static OnceLock<S>;

    fn get_or_init_state<E>(f: impl FnOnce() -> Result<S, E>) -> Result<&'static S, E> {
        let static_ref = Self::get_static();
        let state_opt = static_ref.get();
        match state_opt {
            None => {
                let state = f()?;
                Ok(static_ref.get_or_init(|| state))
            }
            Some(state) => Ok(state),
        }
    }

    #[allow(async_fn_in_trait)]
    async fn get_or_init_state_async<E, F, Fut>(f: F) -> Result<&'static S, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<S, E>>,
    {
        let static_ref = Self::get_static();
        let state_opt = static_ref.get();
        match state_opt {
            None => {
                let state = f().await?;
                Ok(static_ref.get_or_init(|| state))
            }
            Some(state) => Ok(state),
        }
    }

    fn try_state() -> Option<&'static S> {
        Self::get_static().get()
    }

    fn state() -> &'static S {
        Self::try_state().expect("static state should be initialized")
    }
}

pub trait StaticStateMut<S: 'static> {
    fn get_static() -> &'static OnceLock<ArcSwap<S>>;

    fn get_or_init_state<E>(f: impl FnOnce() -> Result<S, E>) -> Result<Arc<S>, E> {
        let static_ref = Self::get_static();
        let asw_opt = static_ref.get();
        let state_arc = match asw_opt {
            None => {
                let state_arc = Arc::new(f()?);
                static_ref.get_or_init(|| ArcSwap::new(state_arc.clone()));
                state_arc
            }
            Some(asw) => asw.load().clone(),
        };
        Ok(state_arc)
    }

    #[allow(async_fn_in_trait)]
    async fn get_or_init_state_async<E, F, Fut>(f: F) -> Result<Arc<S>, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<S, E>>,
    {
        let static_ref = Self::get_static();
        let asw_opt = static_ref.get();
        let state_arc = match asw_opt {
            None => {
                let state_arc = Arc::new(f().await?);
                static_ref.get_or_init(|| ArcSwap::new(state_arc.clone()));
                state_arc
            }
            Some(asw) => asw.load().clone(),
        };
        Ok(state_arc)
    }

    fn update_state(state: S) {
        let static_ref = Self::get_static();
        let asw_opt = static_ref.get();
        match asw_opt {
            None => {
                static_ref.get_or_init(|| ArcSwap::new(state.into()));
            }
            Some(asw) => {
                asw.store(state.into());
            }
        };
    }

    fn try_state() -> Option<Arc<S>> {
        Self::get_static().get().map(|asw| asw.load().clone())
    }

    fn state() -> Arc<S> {
        let asw = Self::get_static()
            .get()
            .expect("static state should be initialized");
        asw.load().clone()
    }
}
