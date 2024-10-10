use app1::{
    run::ctx::Ctx,
    svc::{FooIn, FooOut, FooSfl, FooSflI},
};
use axum::Router;
use foa::{
    db::sqlx::{AsyncTxFn, DbCtx},
    tokio::task_local::{tl_scoped, TaskLocal, TaskLocalCtx},
    web::axum::{default_mapper, HandlerAsyncFn2rsWithErrorMapper},
    Error,
};
use serde::Serialize;
use sqlx::{Postgres, Transaction};
use std::{sync::Arc, time::Duration};

#[derive(Serialize)]
struct FooOutExt {
    foo: FooOut,
    headers: Vec<(String, String)>,
}

struct F;

impl AsyncTxFn for F {
    type In = FooIn;
    type Out = FooOutExt;
    type E = Error;
    type Db = <Ctx as DbCtx>::Db;

    async fn invoke(
        &self,
        input: Self::In,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Self::Out, Self::E> {
        let foo = <FooSflI<Ctx> as FooSfl<Ctx>>::foo_sfl(input, tx).await?;
        let header_map = <Ctx as TaskLocalCtx>::TaskLocal::cloned_value().headers;
        let headers = header_map
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str()))
            .filter(|(_, v)| v.is_ok())
            .map(|(k, v)| (k.to_string(), v.unwrap().to_owned()))
            .collect::<Vec<_>>();
        Ok(FooOutExt { foo, headers })
    }
}

#[tokio::main]
async fn main() {
    Ctx::init().await; // initialize context

    let h = tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            Ctx::refresh_cfg()
                .await
                .expect("Ctx::read_app_cfg_info() error");
        }
    });

    let f = tl_scoped::<_, <Ctx as TaskLocalCtx>::TaskLocal>(F.in_tx());

    let app = Router::new().route(
        "/",
        axum::routing::post(HandlerAsyncFn2rsWithErrorMapper::new(
            Arc::new(f),
            default_mapper,
        )),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    h.await.expect("error joining config refresh task");
}
