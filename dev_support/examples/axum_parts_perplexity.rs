//! Based on https://www.perplexity.ai/search/show-an-axum-post-service-exam-HywavVUyR4Ka1KgsY1OthQ,
//! with corrections and modification to retrieve [`axum::http::request::Parts`]

use axum::{extract::Json, http::request::Parts, routing::post, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Input {
    // Add your input fields here
    name: String,
    age: u32,
}

#[derive(Serialize)]
struct Output {
    // Include all fields from Input
    #[serde(flatten)]
    input: Input,
    // Add the locale field
    locale: String,
}

async fn process_input(parts: Parts, Json(input): Json<Input>) -> Json<Output> {
    let headers = parts.headers;

    // Extract the Accept-Language header
    let locale = headers
        .get("Accept-Language")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("en-US")
        .to_string();

    // Create the output with the input and locale
    let output = Output { input, locale };

    Json(output)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/process", post(process_input));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
