use axum::Router;
use dev_support::artctps::{Ctx, FooSflI};
use foa::web::axum::handler_pg;

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/users",
        axum::routing::post(handler_pg::<Ctx, _, _, _, FooSflI<_>>),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
