use app1::run::{ctx::Ctx, svc_flows::FooSflIC};
use axum::{extract::FromRequestParts, handler::Handler, Router};
// use dev_support::foa_exp::web::axum::{
//     json_handlers_experiment::{direct, from_scratch},
//     json_handlers_old::{handler_asyncfn2r_arc, handler_fn2r},
// };
use foa::{context::ErrCtx, fun::AsyncFn2, web::axum::HandlerAsyncFn2rsWithErrorMapper, Error};
use serde::{de::DeserializeOwned, Serialize};
use std::{sync::Arc, time::Duration};
// use dev_support::foa_exp::web::axum::{
//     json_handlers_experiment::{direct, from_scratch},
//     json_handlers_old::{handler_asyncfn2r_arc, handler_fn2r},
// };
use foa::web::default_mapper;

fn handler<CTX: ErrCtx, O, F, S>(f: F) -> impl Handler<(), S>
where
    F: AsyncFn2<Out = Result<O, Error>> + Send + Sync + 'static + Clone,
    F::In1: FromRequestParts<S>,
    F::In2: DeserializeOwned,
    O: Serialize + Send,
    S: Send + Sync + 'static,
{
    HandlerAsyncFn2rsWithErrorMapper::new(f, default_mapper)
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
        // .route(
        //     "/",
        //     axum::routing::post(HandlerAsyncFn2r(Arc::new(FooSflIC))),
        // )
        // .route(
        //     "/arc",
        //     axum::routing::post(HandlerAsyncFn2rArc::new(FooSflIC)),
        // )
        .route(
            "/",
            axum::routing::post(HandlerAsyncFn2rsWithErrorMapper::new(
                Arc::new(FooSflIC),
                default_mapper,
            )),
        )
        .route(
            "/fn",
            axum::routing::post(handler::<Ctx, _, _, _>(Arc::new(FooSflIC))),
        )
        // .route(
        //     "/scratch",
        //     axum::routing::post(from_scratch::HandlerAsyncFn2r(Arc::new(FooSflIC))),
        // )
        // .route(
        //     "/scratch-arc",
        //     axum::routing::post(from_scratch::HandlerAsyncFn2rArc::new(FooSflIC)),
        // )
        // .route(
        //     "/direct",
        //     axum::routing::post(direct::HandlerAsyncFn2r::<_>::new(Arc::new(FooSflIC)).handler()),
        // )
        // .route(
        //     "/fn",
        //     axum::routing::post(handler_asyncfn2r_arc::<_, _, _, ()>(FooSflIC)),
        // )
        // .route(
        //     "/alt",
        //     axum::routing::post(handler_fn2r::<_, _, _, _, _, ()>(make_foo_sfl())),
        // )
        ;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    h.await.expect("error joining config refresh task");
}
