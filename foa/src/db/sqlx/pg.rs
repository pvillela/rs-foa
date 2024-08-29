use crate::{
    context::DbCtx,
    error::{ErrorKind, FoaError},
};
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

pub trait PgSfl {
    type In;
    type Out;
    type E: From<sqlx::Error>;

    #[allow(async_fn_in_trait)]
    async fn sfl(input: Self::In, tx: &mut Transaction<Postgres>) -> Result<Self::Out, Self::E>;
}

pub async fn pg_sfl<CTX, F>(input: F::In) -> Result<F::Out, F::E>
where
    CTX: DbCtx<Db: Db>,
    F: PgSfl,
{
    let pool = CTX::Db::pool().await?;
    let mut tx = pool.begin().await?;
    let output = F::sfl(input, &mut tx).await?;
    tx.commit().await?;
    Ok(output)
}
