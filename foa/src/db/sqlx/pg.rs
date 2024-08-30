use crate::{
    context::DbCtx,
    error::{ErrorKind, FoaError},
};
use sqlx::{Database, Executor, Pool, Transaction};
use std::future::Future;

// pub trait NiceDb: Database
// where
//     for<'c> &'c mut Self::Connection: Executor<'c, Database = Self>,
// {
// }

// pub trait GoodDb
// // where
// //     for<'c> &'c mut <<Self as GoodDb>::DB as Database>::Connection:
// //         Executor<'c, Database = Self::DB>,
// {
//     type DB: Database
//     where
//         for<'c> &'c mut <<Self as GoodDb>::DB as Database>::Connection:
//             Executor<'c, Database = Self::DB>;
// }

pub trait Db
where
    for<'c> &'c mut <Self::DB as Database>::Connection: Executor<'c, Database = Self::DB>,
{
    type DB: Database;

    fn pool() -> impl Future<Output = Result<Pool<Self::DB>, sqlx::Error>> + Send;
}

// /// Type alias
// pub trait SqlxDbCtx: DbCtx
// where
//     Self::Db: Db,
//     for<'c> &'c mut <<Self::Db as Db>::DB as Database>::Connection:
//         Executor<'c, Database = <Self::Db as Db>::DB>,
// {
// }
// impl<T> SqlxDbCtx for T
// where
//     T: DbCtx,
//     T::Db: Db,
//     for<'c> &'c mut <<T::Db as Db>::DB as Database>::Connection:
//         Executor<'c, Database = <T::Db as Db>::DB>,
// {
//     // type Database = <<T as DbCtx>::Db as Db>::DB;
// }

pub const DB_ERROR: ErrorKind<0, true> = ErrorKind("DB_ERROR", "database error");

impl<CTX> From<sqlx::Error> for FoaError<CTX> {
    fn from(cause: sqlx::Error) -> Self {
        FoaError::new_with_cause_std(&DB_ERROR, cause)
    }
}

pub trait TxSfl<CTX>
where
    CTX: DbCtx<Db: Db>,
    for<'c> &'c mut <<CTX::Db as Db>::DB as Database>::Connection:
        Executor<'c, Database = <CTX::Db as Db>::DB>,
{
    type In;
    type Out;
    type E: From<sqlx::Error>;

    #[allow(async_fn_in_trait)]
    async fn tx_sfl(
        input: Self::In,
        tx: &mut Transaction<<CTX::Db as Db>::DB>,
    ) -> Result<Self::Out, Self::E>;
}

pub async fn txnl_sfl<CTX, F>(input: F::In) -> Result<F::Out, F::E>
where
    CTX: DbCtx<Db: Db>,
    for<'c> &'c mut <<CTX::Db as Db>::DB as Database>::Connection:
        Executor<'c, Database = <CTX::Db as Db>::DB>,
    F: TxSfl<CTX>,
{
    let pool = CTX::Db::pool().await?;
    let mut tx = pool.begin().await?;
    let output = F::tx_sfl(input, &mut tx).await?;
    tx.commit().await?;
    Ok(output)
}
