use crate::error::{ErrorKind, FoaError};
use sqlx::{PgPool, Postgres, Transaction};
use std::future::Future;

pub trait Db {
    fn pool() -> impl Future<Output = Result<PgPool, sqlx::Error>> + Send;
}

pub const DB_ERROR: ErrorKind<0, true> = ErrorKind("DB_ERROR", "database error");

impl<CTX> From<sqlx::Error> for FoaError<CTX> {
    fn from(cause: sqlx::Error) -> Self {
        FoaError::new_with_cause_std(&DB_ERROR, cause)
    }
}

pub trait PgSfl<In, Out> {
    #[allow(async_fn_in_trait)]
    async fn sfl(input: In, tx: &mut Transaction<Postgres>) -> Out;
}

pub async fn pg_sfl<CTX, S, T, E, F>(input: S) -> Result<T, E>
where
    CTX: Db,
    S: 'static + serde::Deserialize<'static>,
    E: From<sqlx::Error>,
    F: PgSfl<S, Result<T, E>>,
{
    let pool = CTX::pool().await?;
    let mut tx = pool.begin().await?;
    let output = F::sfl(input, &mut tx).await?;
    tx.commit().await?;
    Ok(output)
}
