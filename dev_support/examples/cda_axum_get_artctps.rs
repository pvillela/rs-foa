use axum::{routing::post, Router};
use dev_support::artctps::{common::AppCfgInfo, foo_sfl};
use foa::{appcfg::AppCfg, web::axum::handler_of};
use std::{thread, time::Duration};

#[tokio::main]
async fn main() {
    let foo_sfl_hdlr = handler_of(foo_sfl);

    let app = Router::new().route("/", post(foo_sfl_hdlr));

    let _ = thread::spawn(|| loop {
        thread::sleep(Duration::from_millis(500));
        AppCfgInfo::refresh_app_configuration();
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}