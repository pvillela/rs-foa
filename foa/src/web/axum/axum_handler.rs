use crate::db::sqlx::{pg_sfl, Db, PgSfl};
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

pub async fn handler_pg<CTX, F>(Json(input): Json<F::In>) -> Result<Json<F::Out>, F::E>
where
    CTX: Db,
    F: PgSfl,
    F::In: 'static + serde::Deserialize<'static>,
    F::Out: IntoResponse + Send + Sync,
{
    let output = pg_sfl::<CTX, F>(input).await?;
    Ok(Json(output))
}

impl<CTX> IntoResponse for FoaError<CTX> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}
