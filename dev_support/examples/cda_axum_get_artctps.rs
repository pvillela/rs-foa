use std::time::Duration;

use axum::Router;
use dev_support::artctps::{common::Ctx, FooSflI};
use foa::web::axum::handler_pg;

#[tokio::main]
async fn main() {
    Ctx::init().await; // initialize context

    let h = tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            Ctx::refresh_app_cfg_info()
                .await
                .expect("Ctx::read_app_cfg_info() error");
        }
    });

    let app = Router::new().route("/", axum::routing::post(handler_pg::<Ctx, FooSflI<Ctx>>));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    h.await.expect("error joining config refresh task");
}
