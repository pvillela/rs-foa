//! Based on https://www.perplexity.ai/search/show-a-simple-example-of-an-ax-OY34j2O9RFG0mgYLjSKG0A#1,
//! answer to follow-up question, with corrections.

use axum::{extract::State, http::StatusCode, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row =
        sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email")
            .bind(&payload.name)
            .bind(&payload.email)
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = User {
        id: row
            .try_get("id")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        name: row
            .try_get("name")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        email: row
            .try_get("email")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    };

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(user)))
}

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://username:password@localhost/database")
        .await
        .expect("Failed to connect to Postgres");

    let app = Router::new()
        .route("/users", axum::routing::post(create_user))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
