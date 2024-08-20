use super::get_pool;
use futures::Future;
use std::{
    error::Error,
    fmt::{Debug, Display},
};

use super::AppErr;

pub trait Transaction {
    type Tx<'a>;
    type DbErr: Error + Into<AppErr> + Send;

    #[allow(async_fn_in_trait)]
    fn transaction<'a>(
        &'a mut self,
    ) -> impl Future<Output = Result<DummyTx<'a>, Self::DbErr>> + Send;
}

pub trait Db {
    type Db: Transaction + Send;

    #[allow(async_fn_in_trait)]
    fn db_client() -> impl Future<Output = Result<Self::Db, <Self::Db as Transaction>::DbErr>> + Send;
}

pub trait DbCtx {
    type DbClient: Db;
}

pub trait DbClientDefault {}

impl<T> Db for T
where
    T: DbClientDefault,
{
    type Db = DummyDbClient;

    #[allow(async_fn_in_trait)]
    async fn db_client() -> Result<DummyDbClient, DbErr> {
        let pool = get_pool();
        get_connection(pool).await
    }
}

pub struct DummyDbClient;

pub struct DummyDbPool;

#[derive(Debug)]
pub struct DbErr;

impl Display for DbErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Error for DbErr {}

pub trait DbCfg {
    fn get_pool(&self) -> &DummyDbPool;
}

pub async fn get_connection(_pool: &DummyDbPool) -> Result<DummyDbClient, DbErr> {
    // TODO: implement this properly
    Ok(DummyDbClient)
}

pub struct DummyTx<'a> {
    #[allow(unused)]
    db: &'a mut DummyDbClient,
}

impl DummyDbClient {
    pub async fn transaction<'a>(&'a mut self) -> Result<DummyTx<'a>, DbErr> {
        // TODO: implement this properly
        // println!("Db.transaction() called");
        Ok(DummyTx { db: self })
    }
}

impl Transaction for DummyDbClient {
    type Tx<'a> = DummyTx<'a>;
    type DbErr = DbErr;

    async fn transaction<'a>(&'a mut self) -> Result<DummyTx<'a>, Self::DbErr> {
        self.transaction().await
    }
}

impl<'a> DummyTx<'a> {
    pub async fn commit(self) -> Result<(), DbErr> {
        // TODO: implement this properly
        // println!("Tx.commit() called");
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), DbErr> {
        // TODO: implement this properly
        // println!("Tx.rollback() called");
        Ok(())
    }

    /// Dummy method to demonstrate use of transaction reference.
    pub fn dummy(&self, src: &str) -> String {
        format!("-Tx.dummy() called from {}", src)
    }
}
