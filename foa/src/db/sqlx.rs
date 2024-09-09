use crate::{
    error::{ErrorKind, FoaError},
    fun::AsyncRFn,
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
    type E: From<sqlx::Error>;
    type Db: Db;

    #[allow(async_fn_in_trait)]
    fn invoke(
        &self,
        input: Self::In,
        tx: &mut Transaction<<Self::Db as Db>::Database>,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send;

    fn in_tx(
        self,
    ) -> impl AsyncRFn<In = Self::In, Out = Self::Out, E = Self::E> + Send + Sync + 'static
    where
        Self: Send + Sync + Sized + 'static,
    {
        InTxOwned(self)
    }

    fn invoke_in_tx(
        &self,
        input: Self::In,
    ) -> impl std::future::Future<Output = Result<Self::Out, Self::E>> + Send
    where
        Self: Sync + Sized,
    {
        async { InTx(self).invoke(input).await }
    }
}

struct InTx<'a, F>(&'a F);

impl<'a, F> AsyncRFn for InTx<'a, F>
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

struct InTxOwned<F>(F);

impl<F> AsyncRFn for InTxOwned<F>
where
    F: AsyncTxFn + Sync,
{
    type In = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, input: Self::In) -> Result<Self::Out, Self::E> {
        invoke_in_tx(&self.0, input).await
    }
}

pub async fn invoke_in_tx<F>(f: &F, input: F::In) -> Result<F::Out, F::E>
where
    F: AsyncTxFn + Sync,
{
    InTx(f).invoke(input).await
}

pub async fn in_tx_borrowed<'a, F>(
    f: &'a F,
) -> impl AsyncRFn<In = F::In, Out = F::Out, E = F::E> + 'a
where
    F: AsyncTxFn + Sync,
{
    InTx(f)
}

pub async fn in_tx_owned<F>(f: F) -> impl AsyncRFn<In = F::In, Out = F::Out, E = F::E> + 'static
where
    F: AsyncTxFn + Sync + Send + 'static,
{
    InTxOwned(f)
}
