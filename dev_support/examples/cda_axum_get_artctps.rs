use axum::{http::StatusCode, Json, Router};
use dev_support::artctps::{Ctx, FooIn, FooOut, FooSfl, FooSflI};
use foa::{
    db::sqlx::pg::{Db, Itself},
    error::FoaError,
};

async fn foo_sfl_hdlr(
    Json(input): Json<FooIn>,
) -> Result<(StatusCode, Json<FooOut>), FoaError<Ctx>> {
    let ctx = Ctx::itself();
    let mut tx = ctx.pool_tx().await?;
    let foo_out = FooSflI::<Ctx>::foo_sfl(input, &mut tx).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(foo_out)))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/users", axum::routing::post(foo_sfl_hdlr));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
