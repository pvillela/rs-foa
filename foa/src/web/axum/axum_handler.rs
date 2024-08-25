use crate::db::sqlx::pg::{Db, Itself};
use crate::error::FoaError;
use axum::response::IntoResponse;
use axum::Json;
use futures::Future;
use sqlx::{Postgres, Transaction};

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

pub trait PgSfl<In, Out> {
    #[allow(async_fn_in_trait)]
    async fn sfl(input: In, tx: &mut Transaction<Postgres>) -> Out;
}

pub async fn handler_pg<CTX, S, T, E, F>(Json(input): Json<S>) -> Result<Json<T>, E>
where
    CTX: Db + Itself<CTX>,
    S: 'static + serde::Deserialize<'static>,
    T: IntoResponse + Send + Sync,
    E: From<sqlx::Error>,
    F: PgSfl<S, Result<T, E>>,
{
    let ctx = CTX::itself();
    let mut tx = ctx.pool_tx().await?;
    let foo_out = F::sfl(input, &mut tx).await?;
    tx.commit().await?;
    Ok(Json(foo_out))
}

impl<CTX> IntoResponse for FoaError<CTX> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}
