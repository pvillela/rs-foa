use std::time::Duration;

use axum::Router;
use dev_support::artctpg::{common::Ctx, FooSflI};
use foa::{
    fun::Async2RFn,
    trait_utils::Make,
    web::axum::{handler_of_f_headers, handler_tx_headers, handler_tx_headers_fn},
};

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

    let app = Router::new()
        .route(
            "/",
            axum::routing::post(handler_tx_headers::<Ctx, FooSflI<Ctx>, FooSflI<Ctx>, ()>),
        )
        .route(
            "/1",
            axum::routing::post({
                let wf = handler_tx_headers_fn(FooSflI::<Ctx>::make());
                |headers, json| async move { wf.invoke(headers, json).await }
            }),
        )
        .route(
            "/2",
            axum::routing::post(handler_of_f_headers(FooSflI::<Ctx>::make())),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    h.await.expect("error joining config refresh task");
}