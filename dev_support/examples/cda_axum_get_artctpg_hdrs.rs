use axum::Router;
use dev_support::artctpg::{common::Ctx, FooIn, FooOut, FooSfl, FooSflI};
use foa::{
    context::Itself,
    db::sqlx::AsyncTxFn,
    error::FoaError,
    tokio::task_local::{TaskLocal, TaskLocalCtx},
    web::axum::handler_tx_headers,
};
use serde::Serialize;
use sqlx::{Postgres, Transaction};
use std::time::Duration;

#[derive(Serialize)]
struct FooOutExt {
    foo: FooOut,
    headers: Vec<(String, String)>,
}

struct F;

impl Itself for F {
    fn it() -> Self {
        F
    }
}

impl AsyncTxFn<Ctx> for F {
    type In = FooIn;
    type Out = FooOutExt;
    type E = FoaError<Ctx>;

    async fn invoke(
        &self,
        input: Self::In,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Self::Out, Self::E> {
        let foo = FooSflI::<Ctx>::foo_sfl(input, tx).await?;
        let header_map = <Ctx as TaskLocalCtx>::TaskLocal::cloned_value();
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

    let app = Router::new().route("/", axum::routing::post(handler_tx_headers::<Ctx, F, ()>));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    h.await.expect("error joining config refresh task");
}
