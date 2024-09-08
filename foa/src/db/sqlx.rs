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

    fn in_tx(
        self,
    ) -> impl AsyncRFn<In = Self::In, Out = Self::Out, E = Self::E> + Send + Sync + 'static
    where
        CTX: Sync + 'static + Send,
        Self: Send + Sync + Sized + 'static,
    {
        InTxOwned(self, PhantomData)
    }

    fn invoke_in_tx(
        &self,
        input: Self::In,
    ) -> impl std::future::Future<Output = Result<Self::Out, Self::E>> + Send
    where
        CTX: Sync + Send,
        Self: Sync + Sized,
    {
        async { InTx(self, PhantomData).invoke(input).await }
    }
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

struct InTxOwned<CTX, F>(F, PhantomData<CTX>);

impl<CTX, F> AsyncRFn for InTxOwned<CTX, F>
where
    CTX: DbCtx + Sync + Send,
    F: AsyncTxFn<CTX> + Sync,
{
    type In = F::In;
    type Out = F::Out;
    type E = F::E;

    async fn invoke(&self, input: Self::In) -> Result<Self::Out, Self::E> {
        invoke_in_tx(&self.0, input).await
    }
}

pub async fn invoke_in_tx<CTX, F>(f: &F, input: F::In) -> Result<F::Out, F::E>
where
    CTX: DbCtx + Sync,
    F: AsyncTxFn<CTX> + Sync,
{
    InTx(f, PhantomData).invoke(input).await
}

pub async fn in_tx_borrowed<'a, CTX, F>(
    f: &'a F,
) -> impl AsyncRFn<In = F::In, Out = F::Out, E = F::E> + 'a
where
    CTX: DbCtx + Sync + 'a,
    F: AsyncTxFn<CTX> + Sync,
{
    InTx(f, PhantomData)
}

pub async fn in_tx_owned<CTX, F>(
    f: F,
) -> impl AsyncRFn<In = F::In, Out = F::Out, E = F::E> + 'static
where
    CTX: DbCtx + Sync + Send + 'static,
    F: AsyncTxFn<CTX> + Sync + Send + 'static,
{
    InTxOwned(f, PhantomData)
}
