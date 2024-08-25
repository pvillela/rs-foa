use crate::db::sqlx::pg::{pg_sfl, Db, Itself, PgSfl};
use crate::error::FoaError;
use axum::response::IntoResponse;
use axum::Json;
use futures::Future;

pub fn handler_of<S, T, Fut>(
    f: impl Fn(S) -> Fut + 'static + Send + Sync + Clone,
) -> impl Fn(Json<S>) -> Fut + Send + Sync + 'static + Clone
where
    S: 'static + serde::Deserialize<'static>,
    T: IntoResponse + Send + Sync,
    Fut: 'static + Future<Output = T> + Send + Sync,
{
    move |Json(input)| f(input)
}

pub async fn handler_pg<CTX, S, T, E, F>(Json(input): Json<S>) -> Result<Json<T>, E>
where
    CTX: Db + Itself<CTX>,
    S: 'static + serde::Deserialize<'static>,
    T: IntoResponse + Send + Sync,
    E: From<sqlx::Error>,
    F: PgSfl<S, Result<T, E>>,
{
    let output = pg_sfl::<CTX, S, T, E, F>(input).await?;
    Ok(Json(output))
}

impl<CTX> IntoResponse for FoaError<CTX> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}
