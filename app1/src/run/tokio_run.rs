use crate::{
    run::ctx::Ctx,
    svc::{FooIn, FooOut, FooSflI},
};
use axum::http::{request, Method, Uri, Version};
use foa::{
    db::sqlx::AsyncTxFn,
    error::Result,
    fun::AsyncFn2,
    tokio::task_local::{TaskLocal, TaskLocalCtx},
};
use futures::future::join_all;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct FooSflIC;

type CtxTl = <Ctx as TaskLocalCtx>::TaskLocal;
type CtxTlValue = <CtxTl as TaskLocal>::Value;

impl AsyncFn2 for FooSflIC {
    type In1 = CtxTlValue;
    type In2 = FooIn;
    type Out = Result<FooOut>;

    async fn invoke(&self, input1: Self::In1, input2: Self::In2) -> Self::Out {
        FooSflI(Ctx)
            .invoke_in_tx_tl_scoped::<CtxTl>(input1, input2)
            .await
    }
}

pub struct RunIn {
    pub unit_time_millis: u64,
    pub app_cfg_first_refresh_units: u64,
    pub app_cfg_refresh_delta_units: u64,
    pub app_cfg_refresh_count: u64,
    pub increment_to_print: usize,
    pub concurrency: usize,
    pub repeats: usize,
}

pub async fn run(input: RunIn) {
    let RunIn {
        unit_time_millis,
        app_cfg_first_refresh_units,
        app_cfg_refresh_delta_units,
        app_cfg_refresh_count,
        increment_to_print,
        concurrency,
        repeats,
    } = input;

    println!(
        "\n*** run -- {} concurrency, {} repeats",
        concurrency, repeats
    );

    let start_time = Instant::now();
    println!("Started at {:?}", start_time);

    let (parts, _) = {
        let req = request::Builder::new()
            .header("Accept-Language", "pt-BR")
            .method(Method::PUT)
            .uri(Uri::from_static("foo.com"))
            .version(Version::HTTP_2)
            .body(())
            .unwrap();
        req.into_parts()
    };

    // AppCfgInfo::refresh_app_configuration();

    let handle_r = tokio::spawn(async move {
        sleep(Duration::from_millis(
            app_cfg_first_refresh_units * unit_time_millis,
        ))
        .await;
        for _ in 0..app_cfg_refresh_count {
            sleep(Duration::from_millis(
                app_cfg_refresh_delta_units * unit_time_millis,
            ))
            .await;
            Ctx::refresh_cfg()
                .await
                .expect("Ctx::read_app_cfg_info() error");
            println!(
                "App configuration refreshed at elapsed time {:?}.",
                start_time.elapsed()
            );
        }
    });

    let run_concurrent = {
        |i: usize| {
            let parts = parts.clone();
            tokio::spawn(async move {
                let mut res: usize = 0;
                for j in 0..repeats {
                    let out = FooSflIC
                        .invoke(parts.clone(), FooIn { age_delta: 11 })
                        .await;
                    res = format!("{:?}", out).len();
                    if i == 0 && j % increment_to_print == 0 {
                        println!(
                            "foo executed at {:?} elapsed, res={}, out={:?}",
                            start_time.elapsed(),
                            res,
                            out
                        );
                    }
                }
                res
            })
        }
    };

    let handles1 = (0..concurrency).map(run_concurrent).collect::<Vec<_>>();

    println!("about to join handle_r");
    let _ = handle_r
        .await
        .ok()
        .expect("app configuration refresh task failed");
    println!("joined handle_r");

    println!("about to join handles1");
    let res1: usize = join_all(handles1)
        .await
        .iter()
        .map(|x| x.as_ref().ok().expect("Failure in first batch of tasks."))
        .sum();
    println!("joined handles1");

    let average = (res1 as f64) / (concurrency as f64);

    println!(
        "Ended at {:?}, with count={:?}, average={:?}",
        start_time.elapsed(),
        concurrency * repeats,
        average
    );
}
