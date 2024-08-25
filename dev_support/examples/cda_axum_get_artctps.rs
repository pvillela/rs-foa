use axum::{extract::State, http::StatusCode, Json, Router};
use dev_support::artctps::{Ctx, FooIn, FooOut, FooSfl, FooSflI};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

// #[derive(Deserialize)]
// struct CreateUser {
//     name: String,
//     email: String,
// }

// #[derive(Serialize)]
// struct User {
//     id: i32,
//     name: String,
//     email: String,
// }

async fn foo_sfl_hdlr(
    State(pool): State<PgPool>,
    Json(input): Json<FooIn>,
) -> Result<(StatusCode, Json<FooOut>), StatusCode> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // let foo_out = FooOut { res: "done".into() };
    let foo_out = FooSflI::<Ctx>::foo_sfl(input, &mut tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // let row =
    //     sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email")
    //         .bind(&input.name)
    //         .bind(&input.email)
    //         .fetch_one(&mut *tx)
    //         .await
    //         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // let user = User {
    //     id: row
    //         .try_get("id")
    //         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    //     name: row
    //         .try_get("name")
    //         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    //     email: row
    //         .try_get("email")
    //         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    // };

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
