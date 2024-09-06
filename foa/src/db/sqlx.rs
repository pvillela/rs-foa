use crate::{
    error::{ErrorKind, FoaError},
    fun::AsyncRFn,
};
use sqlx::{Database, Pool, Postgres, Transaction};
use std::{future::Future, marker::PhantomData};

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

pub trait AsyncTxFn<CTX>
where
    CTX: DbCtx,
{
    type In: Send;
    type Out: Send;
    type E: From<sqlx::Error>;

    #[allow(async_fn_in_trait)]
    fn invoke(
        &self,
        input: Self::In,
        tx: &mut Transaction<<CTX::Db as Db>::Database>,
    ) -> impl Future<Output = Result<Self::Out, Self::E>> + Send;
}

struct InTx<'a, CTX, F>(&'a F, PhantomData<CTX>);

impl<'a, CTX, F> AsyncRFn for InTx<'a, CTX, F>
where
    CTX: DbCtx + Sync,
    F: AsyncTxFn<CTX> + Sync,
{
    type In = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, input: Self::In) -> Result<Self::Out, Self::E> {
        let pool = CTX::Db::pool().await?;
        let mut tx = pool.begin().await?;
        let output = self.0.invoke(input, &mut tx).await?;
        tx.commit().await?;
        Ok(output)
    }
}

pub async fn in_tx<'a, CTX, F>(f: &'a F) -> impl AsyncRFn<In = F::In, Out = F::Out, E = F::E> + 'a
where
    CTX: DbCtx + Sync + 'static,
    F: AsyncTxFn<CTX> + Sync,
{
    InTx(f, PhantomData)
}

pub async fn invoke_in_tx<CTX, F>(f: &F, input: F::In) -> Result<F::Out, F::E>
where
    CTX: DbCtx + Sync,
    F: AsyncTxFn<CTX> + Sync,
{
    InTx(f, PhantomData).invoke(input).await
}
