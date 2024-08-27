use axum::Router;
use dev_support::artctps::{
    common::{db_pool, Ctx},
    FooSflI,
};
use foa::web::axum::handler_pg;

#[tokio::main]
async fn main() {
    let _ = db_pool().await; // initialize DB_POOL

    let app = Router::new().route(
        "/",
        axum::routing::post(handler_pg::<Ctx, _, _, _, FooSflI<Ctx>>),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
