use axum::{extract::State, http::StatusCode, Json, Router};
use dev_support::artctps::{Ctx, FooIn, FooOut, FooSfl, FooSflI};
use foa::error::FoaError;
use sqlx::PgPool;

async fn foo_sfl_hdlr(
    State(pool): State<PgPool>,
    Json(input): Json<FooIn>,
) -> Result<(StatusCode, Json<FooOut>), FoaError<Ctx>> {
    let mut tx = pool.begin().await?;
    let foo_out = FooSflI::<Ctx>::foo_sfl(input, &mut tx).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(foo_out)))
}

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://username:password@localhost/database")
        .await
        .expect("Failed to connect to Postgres");

    let app = Router::new()
        .route("/users", axum::routing::post(foo_sfl_hdlr))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
