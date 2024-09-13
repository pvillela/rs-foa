use axum::Router;
use dev_support::artctpg::run::{
    ctx::Ctx,
    svc_flows::{make_foo_sfl, FooSflIC},
};
use foa::web::axum::{handler_asyncfn2r_arc, handler_fn2r};
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
            axum::routing::post(handler_asyncfn2r_arc::<_, _, _, ()>(FooSflIC)),
        )
        .route(
            "/alt",
            axum::routing::post(handler_fn2r::<_, _, _, _, _, ()>(make_foo_sfl())),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    h.await.expect("error joining config refresh task");
}
