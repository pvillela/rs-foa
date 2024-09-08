#![allow(deprecated)]

use axum::Router;
use dev_support::artctpg::{common::Ctx, FooSflI};
use foa::{
    trait_utils::Make,
    web::axum::{handler_tx_1requestpart, handler_tx_headers_old},
};
use std::time::Duration;

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
            axum::routing::post(handler_tx_1requestpart::<_, _, _, ()>(
                FooSflI::<Ctx>::make(),
            )),
        )
        .route(
            "/depr",
            axum::routing::post(handler_tx_headers_old::<Ctx, FooSflI<Ctx>, FooSflI<Ctx>>),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    h.await.expect("error joining config refresh task");
}
