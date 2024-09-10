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

    fn invoke(
        &self,
        input: Self::In,
        tx: &mut Transaction<<Self::Db as Db>::Database>,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send;

    fn in_tx<'a>(
        self,
    ) -> impl AsyncRFn<In = Self::In, Out = Self::Out, E = Self::E> + Send + Sync + 'a
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

struct InTx<F>(F);

impl<F> AsyncRFn for InTx<F>
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

pub fn in_tx<'a, F>(f: F) -> impl AsyncRFn<In = F::In, Out = F::Out, E = F::E> + 'a
where
    F: AsyncTxFn + Sync + Send + 'a,
{
    InTx(f)
}

pub async fn invoke_in_tx<F>(f: &F, input: F::In) -> Result<F::Out, F::E>
where
    F: AsyncTxFn + Sync,
{
    InTx(f).invoke(input).await
}
