use super::DbErr;
use axum::response::{IntoResponse, Response};
use derive_more::{Display, Error};
use serde::Serialize;

/// type of application errors.
#[derive(Serialize, Debug, Display, Error)]
pub struct AppErr;

impl From<DbErr> for AppErr {
    fn from(_db_err: DbErr) -> Self {
        // TODO: properly implement this
        AppErr
    }
}

impl IntoResponse for AppErr {
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}
