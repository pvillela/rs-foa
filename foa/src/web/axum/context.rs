use crate::context::LocaleSelf;
use axum::http::request::Parts;

// TODO: needs better implementation that parses the header appropriately
impl LocaleSelf for Parts {
    fn locale(&self) -> Option<&str> {
        self.headers.get("Accept-Language")?.to_str().ok()
    }
}
