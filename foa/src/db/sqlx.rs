use crate::{
    error::{BacktraceSpec, BasicKind, ErrSrcNotTyped, Error, RUNTIME_TAG},
    fun::AsyncFn,
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

pub static DB_ERROR: BasicKind<ErrSrcNotTyped> =
    BasicKind::new("DB_ERROR", Some("database error"), &RUNTIME_TAG)
        .with_backtrace(BacktraceSpec::Env);

impl From<sqlx::Error> for Error {
    fn from(cause: sqlx::Error) -> Self {
        DB_ERROR.error(cause)
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

pub fn in_tx<'a, F>(f: F) -> impl AsyncFn<In = F::In, Out = Result<F::Out, F::E>> + 'a
where
    F: AsyncTxFn + Sync + Send + 'a,
{
    InTx(f)
}

pub async fn invoke_in_tx<F>(f: &F, input: F::In) -> Result<F::Out, F::E>
where
    F: AsyncTxFn + Sync,
{
    f.in_tx().invoke(input).await
}
