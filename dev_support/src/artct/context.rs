use super::common::{AppErr, Db, DbCtx, DummyTx, Transaction};

pub trait Cfg {
    type Info;

    fn cfg() -> Self::Info;
}

pub trait CfgCtx {
    type Cfg: Cfg;
}

pub trait AsyncFnTx<CTX, IN, OUT>
where
    CTX: DbCtx,
{
    #[allow(async_fn_in_trait)]
    async fn f(input: IN, tx: &DummyTx<'_>) -> Result<OUT, AppErr>;

    #[allow(async_fn_in_trait)]
    async fn exec_with_transaction(input: IN) -> Result<OUT, AppErr> {
        // let pool = get_pool();
        // let mut db = get_connection(pool).await?;
        let mut db = CTX::DbClient::db_client().await.map_err(|err| err.into())?;
        let tx: DummyTx = db.transaction().await.map_err(|err| err.into())?;

        let res = Self::f(input, &tx).await;
        if res.is_ok() {
            tx.commit().await?;
        } else {
            tx.rollback().await?;
        }
        res
    }
}
