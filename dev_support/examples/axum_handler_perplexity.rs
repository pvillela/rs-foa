//! From https://www.perplexity.ai/search/show-an-example-of-a-custom-im-jTOswStFQC.0SsL9I2O1ZQ,
//! answer to follow-up question, with corrections.

use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    handler::Handler,
    response::IntoResponse,
    response::Response,
};
use std::future::Future;
use std::pin::Pin;

// Wrapper struct for our handler function
#[derive(Clone)]
struct W<F>(F);

impl<M, T1, T2, S, F, Fut, Res> Handler<(M, T1, T2), S> for W<F>
where
    F: FnOnce(T1, T2) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Res> + Send,
    S: Send + Sync + 'static,
    Res: IntoResponse,
    T1: FromRequestParts<S> + Send,
    T2: FromRequest<S, M> + Send,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, req: Request, state: S) -> Self::Future {
        Box::pin(async move {
            // Split the request into parts and body
            let (mut parts, body) = req.into_parts();

            // Extract T1 from request parts
            let t1 = match T1::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };

            // Reconstruct the request
            let req = Request::from_parts(parts, body);

            // Extract T2 from the full request
            let t2 = match T2::from_request(req, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };

            // Call the wrapped function with extracted values
            let result = (self.0)(t1, t2).await;

            // Convert the result to a response
            result.into_response()
        })
    }
}

// Helper function to create our wrapped handler
fn handler<F, T1, T2, Fut, Res>(f: F) -> W<F>
where
    F: FnOnce(T1, T2) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Res> + Send,
    Res: IntoResponse,
{
    W(f)
}

// Example usage
use axum::{extract::Path, routing::get, Router};

async fn my_handler(Path(id): Path<u32>, body: String) -> String {
    format!("ID: {}, Body: {}", id, body)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/:id", get(handler(my_handler)));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
