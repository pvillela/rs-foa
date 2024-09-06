use std::time::Duration;

use axum::{http::HeaderMap, Json, Router};
use dev_support::artctpg::{common::Ctx, FooSflI};
use foa::{
    context::Itself,
    db::sqlx::{AsyncTxFn, DbCtx},
    fun::Async2RFn,
    tokio::task_local::{TaskLocal, TaskLocalCtx},
    web::axum::{
        handler, handler_of_2r_oj, handler_tx_fn, handler_tx_headers, pre_hdlr_tx_headers,
    },
};
use serde::{Deserialize, Serialize};

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

    let f1 = pre_hdlr_tx_headers(FooSflI::<Ctx>::it());

    let handler = handler_of_2r_oj(f1);

    let app = Router::new().route("/", axum::routing::post(handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    h.await.expect("error joining config refresh task");
}
