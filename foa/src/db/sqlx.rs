use crate::{
    error::{ErrorKind, FoaError},
    tokio::task_local::{TaskLocal, TaskLocalCtx},
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

pub trait AsyncTxFn<CTX>
where
    CTX: DbCtx,
{
    type In;
    type Out;
    type E: From<sqlx::Error>;

    #[allow(async_fn_in_trait)]
    async fn call(
        input: Self::In,
        tx: &mut Transaction<<CTX::Db as Db>::Database>,
    ) -> Result<Self::Out, Self::E>;

    #[allow(async_fn_in_trait)]
    async fn in_tx(input: Self::In) -> Result<Self::Out, Self::E> {
        let pool = CTX::Db::pool().await?;
        let mut tx = pool.begin().await?;
        let output = Self::call(input, &mut tx).await?;
        tx.commit().await?;
        Ok(output)
    }
}

pub trait AsyncTlTxFn<CTX, D = ()>
where
    CTX: DbCtx + TaskLocalCtx<D>,
{
    type In;
    type Out;
    type E: From<sqlx::Error>;

    #[allow(async_fn_in_trait)]
    async fn call(
        input: Self::In,
        tx: &mut Transaction<<CTX::Db as Db>::Database>,
    ) -> Result<Self::Out, Self::E>;

    #[allow(async_fn_in_trait)]
    async fn tl_scoped_in_tx(
        value: <CTX::TaskLocal as TaskLocal<D>>::ValueType,
        input: Self::In,
    ) -> Result<Self::Out, Self::E> {
        let pool = CTX::Db::pool().await?;
        let mut tx = pool.begin().await?;
        let lk = CTX::TaskLocal::local_key();
        let output = lk.scope(value, Self::call(input, &mut tx)).await?;
        tx.commit().await?;
        Ok(output)
    }
}
