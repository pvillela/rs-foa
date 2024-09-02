use std::time::Duration;

use axum::Router;
use dev_support::artctps::{common::Ctx, FooIn, FooOut, FooSfl, FooSflI};
use foa::{
    db::sqlx::{AsyncTlTxFn, Db},
    error::FoaError,
    tokio::task_local::TaskLocalCtx,
    web::axum::handler_tx_headers,
};
use serde::Serialize;
use sqlx::Transaction;

#[derive(Serialize)]
struct FooOutExt {
    foo: FooOut,
    headers: Vec<(String, String)>,
}

struct F;

impl AsyncTlTxFn<Ctx> for F {
    type In = FooIn;
    type Out = FooOutExt;
    type E = FoaError<Ctx>;

    async fn call(
        input: Self::In,
        tx: &mut Transaction<'_, <Ctx as Db>::Database>,
    ) -> Result<Self::Out, Self::E> {
        let foo = FooSflI::<Ctx>::foo_sfl(input, tx).await?;
        let header_map = Ctx::tl_value();
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
