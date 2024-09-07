use arc_swap::ArcSwap;
use std::{
    future::Future,
    sync::{Arc, OnceLock},
};

/// Represents immutable state stored in a static variable.
///
/// Without loss of generality, if a type `T` needs to implement [`StaticState`] for [`State`](StaticState::State)
/// types `S1` and `S2`, then `T` can implement `StaticState` with `type State = (S1, S2)`.
pub trait StaticState {
    type State: 'static;

    fn get_static() -> &'static OnceLock<Self::State>;

    fn get_or_init_state<E>(
        f: impl FnOnce() -> Result<Self::State, E>,
    ) -> Result<&'static Self::State, E> {
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
    async fn get_or_init_state_async<E, F, Fut>(f: F) -> Result<&'static Self::State, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Self::State, E>>,
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

    fn try_state() -> Option<&'static Self::State> {
        Self::get_static().get()
    }

    fn state() -> &'static Self::State {
        Self::try_state().expect("static state should be initialized")
    }
}

/// Represents mutable state stored in a static variable.
///
/// Without loss of generality, if a type `T` needs to implement [`StaticStateMut`] for [`State`](StaticStateMut::State)
/// types `S1` and `S2`, then `T` can implement `StaticState` with `type State = (S1, S2)`.
pub trait StaticStateMut {
    type State: 'static;

    fn get_static() -> &'static OnceLock<ArcSwap<Self::State>>;

    fn get_or_init_state<E>(
        f: impl FnOnce() -> Result<Self::State, E>,
    ) -> Result<Arc<Self::State>, E> {
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
    async fn get_or_init_state_async<E, F, Fut>(f: F) -> Result<Arc<Self::State>, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Self::State, E>>,
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

    fn update_state(state: Self::State) {
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

    fn try_state() -> Option<Arc<Self::State>> {
        Self::get_static().get().map(|asw| asw.load().clone())
    }

    fn state() -> Arc<Self::State> {
        let asw = Self::get_static()
            .get()
            .expect("static state should be initialized");
        asw.load().clone()
    }
}
