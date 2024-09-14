#[allow(deprecated)]
use crate::tokio::task_local::tl_scoped_old;
use crate::{
    error::{ErrorKind, FoaError},
    fun::{AsyncFn, AsyncFn2, AsyncRFn, AsyncRFn2},
    tokio::task_local::{invoke_tl_scoped, tl_scoped, TaskLocal},
};

use sqlx::{Database, Pool, Postgres, Transaction};
use std::future::Future;

pub trait DbCtx {
    type Db: Db;
}

pub trait Db {
    type Database: Database;

    fn pool() -> impl Future<Output = Result<Pool<Self::Database>, sqlx::Error>> + Send;
}

/// Type alias
pub trait PgDbCtx: DbCtx<Db: Db<Database = Postgres>> {}
impl<T> PgDbCtx for T where T: DbCtx<Db: Db<Database = Postgres>> {}

pub const DB_ERROR: ErrorKind<0, true> = ErrorKind("DB_ERROR", "database error");

impl<CTX> From<sqlx::Error> for FoaError<CTX> {
    fn from(cause: sqlx::Error) -> Self {
        FoaError::new_with_cause_std(&DB_ERROR, cause)
    }
}

pub trait AsyncTxFn {
    type In: Send;
    type Out: Send;
    type E: From<sqlx::Error> + Send;
    type Db: Db;

    fn invoke(
        &self,
        input: Self::In,
        tx: &mut Transaction<<Self::Db as Db>::Database>,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send;

    #[deprecated]
    #[allow(deprecated)]
    fn in_tx_old<'a>(
        self,
    ) -> impl AsyncRFn<In = Self::In, Out = Self::Out, E = Self::E> + Send + Sync + 'a
    where
        Self: Send + Sync + Sized + 'a,
    {
        in_tx_old(self)
    }

    fn in_tx<'a>(
        self,
    ) -> impl AsyncFn<In = Self::In, Out = Result<Self::Out, Self::E>> + Send + Sync + 'a
    where
        Self: Send + Sync + Sized + 'a,
    {
        in_tx(self)
    }

    #[allow(async_fn_in_trait)]
    async fn invoke_in_tx(&self, input: Self::In) -> Result<Self::Out, Self::E>
    where
        Self: Sync + Sized,
    {
        invoke_in_tx(self, input).await
    }

    #[deprecated]
    #[allow(deprecated)]
    fn in_tx_tl_scoped_old<'a, TL>(
        self,
    ) -> impl AsyncRFn2<In1 = TL::Value, In2 = Self::In, Out = Self::Out, E = Self::E> + Send + Sync + 'a
    where
        Self: Send + Sync + Sized + 'a,
        TL: TaskLocal + Sync + Send + 'static,
        TL::Value: Send,
    {
        in_tx_tl_scoped_old::<_, TL>(self)
    }

    fn in_tx_tl_scoped<'a, TL>(
        self,
    ) -> impl AsyncFn2<In1 = TL::Value, In2 = Self::In, Out = Result<Self::Out, Self::E>>
           + Send
           + Sync
           + 'a
    where
        Self: Send + Sync + Sized + 'a,
        TL: TaskLocal + Sync + Send + 'static,
        TL::Value: Send,
    {
        in_tx_tl_scoped::<_, TL>(self)
    }

    #[allow(async_fn_in_trait)]
    async fn invoke_in_tx_tl_scoped<TL>(
        &self,
        in1: TL::Value,
        in2: Self::In,
    ) -> Result<Self::Out, Self::E>
    where
        Self: Sync + Sized,
        TL: TaskLocal + Sync + 'static,
        TL::Value: Send,
    {
        invoke_in_tx_tl_scoped::<_, TL>(self, in1, in2).await
    }
}

impl<F: AsyncTxFn> AsyncTxFn for &F {
    type In = F::In;
    type Out = F::Out;
    type E = F::E;
    type Db = F::Db;

    fn invoke(
        &self,
        input: Self::In,
        tx: &mut Transaction<<Self::Db as Db>::Database>,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send {
        F::invoke(self, input, tx)
    }
}

#[deprecated]
struct InTxOld<F>(F);

#[allow(deprecated)]
impl<F> AsyncRFn for InTxOld<F>
where
    F: AsyncTxFn + Sync,
{
    type In = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, input: Self::In) -> Result<Self::Out, Self::E> {
        let pool = F::Db::pool().await?;
        let mut tx = pool.begin().await?;
        let output = self.0.invoke(input, &mut tx).await?;
        tx.commit().await?;
        Ok(output)
    }
}

struct InTx<F>(F);

impl<F> AsyncFn for InTx<F>
where
    F: AsyncTxFn + Sync,
{
    type In = F::In;
    type Out = Result<F::Out, F::E>;

    async fn invoke(&self, input: Self::In) -> Self::Out {
        let pool = F::Db::pool().await?;
        let mut tx = pool.begin().await?;
        let output = self.0.invoke(input, &mut tx).await?;
        tx.commit().await?;
        Ok(output)
    }
}

#[deprecated]
#[allow(deprecated)]
pub fn in_tx_old<'a, F>(f: F) -> impl AsyncRFn<In = F::In, Out = F::Out, E = F::E> + 'a
where
    F: AsyncTxFn + Sync + Send + 'a,
{
    InTxOld(f)
}

pub fn in_tx<'a, F>(f: F) -> impl AsyncFn<In = F::In, Out = Result<F::Out, F::E>> + 'a
where
    F: AsyncTxFn + Sync + Send + 'a,
{
    InTx(f)
}

#[deprecated]
#[allow(deprecated)]
pub async fn invoke_in_tx_old<F>(f: &F, input: F::In) -> Result<F::Out, F::E>
where
    F: AsyncTxFn + Sync,
{
    AsyncRFn::invoke(&InTxOld(f), input).await
}

pub async fn invoke_in_tx<F>(f: &F, input: F::In) -> Result<F::Out, F::E>
where
    F: AsyncTxFn + Sync,
{
    f.in_tx().invoke(input).await
}

#[deprecated]
#[allow(deprecated)]
pub fn in_tx_tl_scoped_old<'a, F, TL>(
    f: F,
) -> impl AsyncRFn2<In1 = TL::Value, In2 = F::In, Out = F::Out, E = F::E> + 'a
where
    TL: TaskLocal + Sync + 'static,
    TL::Value: Send,
    F: AsyncTxFn + Sync + Send + 'a,
{
    let wf1 = f.in_tx_old();
    tl_scoped_old::<_, TL>(wf1)
}

pub fn in_tx_tl_scoped<'a, F, TL>(
    f: F,
) -> impl AsyncFn2<In1 = TL::Value, In2 = F::In, Out = Result<F::Out, F::E>> + 'a
where
    TL: TaskLocal + Sync + 'static,
    TL::Value: Send,
    F: AsyncTxFn + Sync + Send + 'a,
{
    let wf1 = f.in_tx();
    tl_scoped::<_, TL>(wf1)
}

#[deprecated]
pub async fn invoke_in_tx_tl_scoped_old<F, TL>(
    f: &F,
    in1: TL::Value,
    in2: F::In,
) -> Result<F::Out, F::E>
where
    TL: TaskLocal + Sync + 'static,
    TL::Value: Send,
    F: AsyncTxFn + Sync,
{
    let wf1 = f.in_tx();
    invoke_tl_scoped::<_, TL>(&wf1, in1, in2).await
}

pub async fn invoke_in_tx_tl_scoped<F, TL>(
    f: &F,
    in1: TL::Value,
    in2: F::In,
) -> Result<F::Out, F::E>
where
    TL: TaskLocal + Sync + 'static,
    TL::Value: Send,
    F: AsyncTxFn + Sync,
{
    let wf1 = f.in_tx();
    invoke_tl_scoped::<_, TL>(&wf1, in1, in2).await
}
