use std::time::Duration;

use axum::Router;
use dev_support::artctps::{common::Ctx, FooIn, FooOut, FooSfl, FooSflI};
use foa::{
    db::sqlx::{AsyncTlTxFn, Db},
    error::FoaError,
    tokio::task_local::TaskLocalCtx,
    web::axum::handler_tx_headers,
};
use sqlx::Transaction;

struct F;

impl AsyncTlTxFn<Ctx> for F {
    type In = FooIn;
    type Out = FooOut;
    type E = FoaError<Ctx>;

    async fn call(
        input: Self::In,
        tx: &mut Transaction<'_, <Ctx as Db>::Database>,
    ) -> Result<Self::Out, Self::E> {
        let mut foo_out = FooSflI::<Ctx>::foo_sfl(input, tx).await?;
        let headers = Ctx::tl_value();
        foo_out.name = format!("{:?}", headers);
        Ok(foo_out)
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
