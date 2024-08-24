use crate::error::{ErrorKind, FoaError};
use sqlx::{Error as SqlxError, PgConnection, Postgres, Transaction};
use std::{
    fmt::{Debug, Display},
    future::Future,
};

// pub trait Transaction<CTX> {
//     type Tx<'a>;
//     type DbErr: Error + Into<FoaError<CTX>> + Send;

//     #[allow(async_fn_in_trait)]
//     fn transaction<'a>(
//         &'a mut self,
//     ) -> impl Future<Output = Result<DummyTx<'a>, Self::DbErr>> + Send;
// }

// pub trait Db {
//     type Db: Transaction + Send;

//     #[allow(async_fn_in_trait)]
//     fn db_client() -> impl Future<Output = Result<Self::Db, <Self::Db as Transaction>::DbErr>> + Send;
// }

// pub trait DbCtx {
//     type DbClient: Db;
// }

// pub trait DbClientDefault {}

// impl<T> Db for T
// where
//     T: DbClientDefault,
// {
//     type Db = DummyDbClient;

//     #[allow(async_fn_in_trait)]
//     async fn db_client() -> Result<DummyDbClient, DbErr> {
//         let pool = get_pool();
//         get_connection(pool).await
//     }
// }

// pub struct DummyDbClient;

// pub struct DummyDbPool;

// #[derive(Debug)]
// pub struct DbErr;

// impl Display for DbErr {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         Debug::fmt(&self, f)
//     }
// }

// impl Error for DbErr {}

// pub trait DbCfg {
//     fn get_pool(&self) -> &DummyDbPool;
// }

// pub async fn get_connection(_pool: &DummyDbPool) -> Result<DummyDbClient, DbErr> {
//     // TODO: implement this properly
//     Ok(DummyDbClient)
// }

// pub struct DummyTx<'a> {
//     #[allow(unused)]
//     db: &'a mut DummyDbClient,
// }

// impl DummyDbClient {
//     pub async fn transaction<'a>(&'a mut self) -> Result<DummyTx<'a>, DbErr> {
//         // TODO: implement this properly
//         // println!("Db.transaction() called");
//         Ok(DummyTx { db: self })
//     }
// }

// impl Transaction for DummyDbClient {
//     type Tx<'a> = DummyTx<'a>;
//     type DbErr = DbErr;

//     async fn transaction<'a>(&'a mut self) -> Result<DummyTx<'a>, Self::DbErr> {
//         self.transaction().await
//     }
// }

// impl<'a> DummyTx<'a> {
//     pub async fn commit(self) -> Result<(), DbErr> {
//         // TODO: implement this properly
//         // println!("Tx.commit() called");
//         Ok(())
//     }

//     pub async fn rollback(self) -> Result<(), DbErr> {
//         // TODO: implement this properly
//         // println!("Tx.rollback() called");
//         Ok(())
//     }

//     /// Dummy method to demonstrate use of transaction reference.
//     pub fn dummy(&self, src: &str) -> String {
//         format!("-Tx.dummy() called from {}", src)
//     }
// }

pub trait Db {
    #[allow(async_fn_in_trait)]
    fn pool_tx<'c>(
        &'c self,
    ) -> impl Future<Output = Result<Transaction<'c, Postgres>, SqlxError>> + Send;
}

pub trait Itself<CTX> {
    fn itself() -> CTX;
}

pub const DB_ERROR: ErrorKind<0, true> = ErrorKind("DB_ERROR", "database error");

impl<CTX> From<SqlxError> for FoaError<CTX> {
    fn from(cause: SqlxError) -> Self {
        FoaError::new_with_cause_std(&DB_ERROR, cause)
    }
}

pub trait AsyncFnTx<CTX, IN, OUT>
where
    CTX: Db + Itself<CTX>,
{
    #[allow(async_fn_in_trait)]
    async fn f(input: IN, tx: &mut PgConnection) -> Result<OUT, FoaError<CTX>>;

    #[allow(async_fn_in_trait)]
    async fn exec_with_transaction(input: IN) -> Result<OUT, FoaError<CTX>> {
        let ctx = CTX::itself();
        let mut tx = ctx.pool_tx().await?;
        let res = Self::f(input, &mut *tx).await;
        if res.is_ok() {
            tx.commit().await?;
        } else {
            tx.rollback().await?;
        }
        res
    }
}
