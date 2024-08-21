use axum::{routing::post, Router};
use dev_support::artct::common::{handler_of, refresh_app_configuration};
use dev_support::artct::foo_artct_sfl;
use std::{thread, time::Duration};

#[tokio::main]
async fn main() {
    let foo_artct_sfl_hdlr = handler_of(foo_artct_sfl);

    let app = Router::new().route("/", post(foo_artct_sfl_hdlr));

    let _ = thread::spawn(|| loop {
        thread::sleep(Duration::from_millis(500));
        refresh_app_configuration();
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}