use std::{
    cell::Cell,
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
    thread::{self, ThreadId},
};

struct UnsafeSyncCell<V>(Cell<V>);

/// SAFETY: An instance is only accessed privately in [`ThreadMap`], always in the same thread.
unsafe impl<V> Sync for UnsafeSyncCell<V> {}

impl<V: Debug> Debug for UnsafeSyncCell<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", unsafe { &*self.0.as_ptr() }))
    }
}

/// This type encapsulates the association of [`ThreadId`]s to values of type `V`. It is a simple and easy-to-use alternative
/// to the [`std::thread_local`] macro and the [`thread_local`](https://crates.io/crates/thread_local) crate.
#[derive(Debug)]
pub struct ThreadMap<V> {
    state: Arc<RwLock<HashMap<ThreadId, UnsafeSyncCell<V>>>>,
    value_constr: fn() -> V,
}

impl<V> ThreadMap<V> {
    /// Creates a new [`ThreadMap`] instance, with `value_constr` used to instantiate the initial value for each thread.
    pub fn new(value_constr: fn() -> V) -> Self {
        Self {
            state: RwLock::new(HashMap::new()).into(),
            value_constr,
        }
    }

    /// Invokes `f` on the value associated with the [`ThreadId`] of the current thread and returns the invocation result.
    /// If there is no value associated with the current thread then the `value_constr` argument of [`Self::new`] is used
    /// to instantiate an initial associated value before `f` is applied.
    pub fn with<W>(&self, f: impl FnOnce(&mut V) -> W) -> W {
        let lock = self.state.read().expect("unable to get read lock");
        let tid = thread::current().id();
        match lock.get(&tid) {
            Some(c) => {
                let v = c.0.as_ptr();
                // SAFETY: call below is always done in the thread with `ThreadId` `tid`.
                let rv = unsafe { &mut *v };
                f(rv)
            }
            None => {
                drop(lock);
                let mut lock = self.state.write().expect("unable to get write lock");
                let mut v0 = (self.value_constr)();
                let w = f(&mut v0);
                lock.insert(tid, UnsafeSyncCell(Cell::new(v0)));
                w
            }
        }
    }

    /// Returns a [`HashMap`] with the values associated with each [`ThreadId`] key.
    ///
    /// # Errors
    /// - `Some(self)` if there are active cloned instances
    /// - `None` if the internal lock is poisoned
    pub fn try_dump(self) -> Result<HashMap<ThreadId, V>, Option<Self>> {
        match Arc::try_unwrap(self.state) {
            Ok(rwlock) => match rwlock.into_inner() {
                Ok(inner) => {
                    let map: HashMap<ThreadId, V> = inner
                        .into_iter()
                        .map(|(k, v)| (k, v.0.into_inner()))
                        .collect();
                    Ok(map)
                }
                Err(_e) => Err(None),
            },
            Err(arc) => Err(Some(Self {
                state: arc,
                value_constr: self.value_constr,
            })),
        }
    }

    /// Returns a [`HashMap`] with clones of the values associated with each [`ThreadId`] key at the time the probe
    /// was executed; returns `None` if the internal lock is poisoned.
    pub fn probe(&self) -> Option<HashMap<ThreadId, V>>
    where
        V: Clone,
    {
        let map = self
            .state
            .write()
            .ok()?
            .iter()
            .map(|(tid, v)| {
                let rv = unsafe { &*v.0.as_ptr() };
                (*tid, rv.clone())
            })
            .collect::<HashMap<_, _>>();

        Some(map)
    }
}

impl<V> Clone for ThreadMap<V> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            value_constr: self.value_constr,
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::HashMap,
        thread::{self},
        time::Duration,
    };

    use super::ThreadMap;

    const NTHREADS: i32 = 20;
    const NITER: i32 = 10;
    const SLEEP_MICROS: u64 = 10;

    fn value_constr() -> (i32, i32) {
        (0, 0)
    }

    fn g((i0, v0): &mut (i32, i32), i: i32) {
        *i0 = i;
        *v0 += i;
    }

    #[test]
    fn test() {
        let tm = ThreadMap::new(value_constr);

        thread::scope(|s| {
            for i in 0..NTHREADS {
                let f = move |p: &mut (i32, i32)| g(p, i);
                let tm = &tm;
                s.spawn(move || {
                    let tm = tm.clone();
                    for _ in 0..NITER {
                        thread::sleep(Duration::from_micros(SLEEP_MICROS));
                        tm.with(f)
                    }
                });
            }

            let probed = tm.probe().unwrap().into_values().collect::<HashMap<_, _>>();
            println!("probed={probed:?}");

            let f = move |p: &mut (i32, i32)| g(p, NTHREADS);
            for _ in 0..NITER {
                tm.with(f)
            }

            let probed = tm.probe().unwrap().into_values().collect::<HashMap<_, _>>();
            println!("probed={probed:?}");
        });

        let expected = (0..=NTHREADS)
            .map(|i| (i, i * NITER))
            .collect::<HashMap<_, _>>();

        let probed = tm.probe().unwrap().into_values().collect::<HashMap<_, _>>();
        println!("probed={probed:?}");

        assert_eq!(expected, probed);

        let dumped = tm
            .try_dump()
            .unwrap()
            .into_values()
            .collect::<HashMap<_, _>>();

        assert_eq!(expected, dumped);
    }
}
