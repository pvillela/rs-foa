pub use foo_a_data::*;
pub use foo_art_data::*;

mod foo_a_data {
    use axum;
    use axum::response::{IntoResponse, Response};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Deserialize, Debug)]
    pub struct FooAIn {
        pub sleep_millis: u64,
    }

    #[allow(unused)]
    #[derive(Serialize, Debug)]
    pub struct FooAOut {
        pub res: String,
    }

    impl IntoResponse for FooAOut {
        fn into_response(self) -> Response {
            axum::Json(self).into_response()
        }
    }
}

mod foo_art_data {
    use super::{FooAIn, FooAOut};

    pub type FooArtIn = FooAIn;
    pub type FooArtOut = FooAOut;
}
