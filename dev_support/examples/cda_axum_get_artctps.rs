use axum::Router;
use dev_support::artctps::{common::Ctx, FooSflI};
use foa::web::axum::handler_pg;

#[tokio::main]
async fn main() {
    Ctx::init().await; // initialize context

    let app = Router::new().route(
        "/",
        axum::routing::post(handler_pg::<Ctx, _, _, _, FooSflI<Ctx>>),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
