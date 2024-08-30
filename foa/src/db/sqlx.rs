use crate::error::{ErrorKind, FoaError};
use sqlx::{Database, Pool, Postgres, Transaction};
use std::future::Future;

pub trait Db {
    type Database: Database;

    fn pool() -> impl Future<Output = Result<Pool<Self::Database>, sqlx::Error>> + Send;
}

/// Type alias
pub trait PgDb: Db<Database = Postgres> {}
impl<T> PgDb for T where T: Db<Database = Postgres> {}

pub const DB_ERROR: ErrorKind<0, true> = ErrorKind("DB_ERROR", "database error");

impl<CTX> From<sqlx::Error> for FoaError<CTX> {
    fn from(cause: sqlx::Error) -> Self {
        FoaError::new_with_cause_std(&DB_ERROR, cause)
    }
}

pub trait AsyncTxFn<CTX>
where
    CTX: Db,
{
    type In;
    type Out;
    type E: From<sqlx::Error>;

    #[allow(async_fn_in_trait)]
    async fn call(
        input: Self::In,
        tx: &mut Transaction<CTX::Database>,
    ) -> Result<Self::Out, Self::E>;

    #[allow(async_fn_in_trait)]
    async fn in_tx(input: Self::In) -> Result<Self::Out, Self::E> {
        let pool = CTX::pool().await?;
        let mut tx = pool.begin().await?;
        let output = Self::call(input, &mut tx).await?;
        tx.commit().await?;
        Ok(output)
    }
}
