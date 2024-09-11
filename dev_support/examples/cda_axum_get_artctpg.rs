use axum::Router;
use dev_support::artctpg::{common::Ctx, FooSflI};
use foa::{db::sqlx::AsyncTxFn, tokio::task_local::TaskLocalCtx, web::axum::handler_asyncrfn2_arc};
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

    let foo_sfl = FooSflI(Ctx).in_tx_tl_scoped::<<Ctx as TaskLocalCtx>::TaskLocal>();

    let app = Router::new().route(
        "/",
        axum::routing::post(handler_asyncrfn2_arc::<_, ()>(foo_sfl)),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    h.await.expect("error joining config refresh task");
}
