use axum::Router;
use dev_support::artctpg::run::{
    ctx::Ctx,
    svc_flows::{make_foo_sfl, FooSflIC},
};
use dev_support::foa_exp::web::axum::json_handlers_experiment::{direct, from_scratch};
use foa::{
    fun::{AsyncFn, AsyncFn2},
    web::axum::{
        handler_asyncfn2r_arc, handler_fn2r, HandlerAsyncFn2r, HandlerAsyncFn2rArc,
        HandlerAsyncFn2rWithErrorMapper,
    },
};
use std::{sync::Arc, time::Duration};

type FooOut = <FooSflIC as AsyncFn2>::Out;

#[derive(Clone)]
struct IdentityErrorMapper;

impl AsyncFn for IdentityErrorMapper {
    type In = FooOut;
    type Out = FooOut;

    async fn invoke(&self, input: Self::In) -> Self::Out {
        input
    }
}

#[tokio::main]
async fn main() {
    Ctx::init().await; // initialize context

    let h = tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            Ctx::refresh_cfg()
                .await
                .expect("Ctx::read_app_cfg_info() error");
        }
    });

    let app = Router::new()
        .route(
            "/",
            axum::routing::post(HandlerAsyncFn2r(Arc::new(FooSflIC))),
        )
        .route(
            "/arc",
            axum::routing::post(HandlerAsyncFn2rArc::new(FooSflIC)),
        )
        .route(
            "/mapped",
            axum::routing::post(HandlerAsyncFn2rWithErrorMapper(
                Arc::new(FooSflIC),
                IdentityErrorMapper,
            )),
        )
        .route(
            "/scratch",
            axum::routing::post(from_scratch::HandlerAsyncFn2r(Arc::new(FooSflIC))),
        )
        .route(
            "/scratch-arc",
            axum::routing::post(from_scratch::HandlerAsyncFn2rArc::new(FooSflIC)),
        )
        .route(
            "/direct",
            axum::routing::post(direct::HandlerAsyncFn2r::<_>::new(Arc::new(FooSflIC)).handler()),
        )
        .route(
            "/fn",
            axum::routing::post(handler_asyncfn2r_arc::<_, _, _, ()>(FooSflIC)),
        )
        .route(
            "/alt",
            axum::routing::post(handler_fn2r::<_, _, _, _, _, ()>(make_foo_sfl())),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    h.await.expect("error joining config refresh task");
}
