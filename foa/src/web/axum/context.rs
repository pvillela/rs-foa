use crate::context::LocaleSelf;
use axum::http::request::Parts;

impl LocaleSelf for Parts {
    fn locale(&self) -> Option<&str> {
        self.headers.get("Accept-Language")?.to_str().ok()
    }
}
