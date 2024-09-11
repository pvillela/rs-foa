use crate::artctpg::svc::{common::Ctx, FooIn, FooSflI};
use foa::db::sqlx::invoke_in_tx;
use futures::future::join_all;
use std::time::{Duration, Instant};
use tokio::time::sleep;

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
            tokio::spawn(async move {
                let foo_sfl = FooSflI(Ctx);
                let mut res: usize = 0;
                for j in 0..repeats {
                    let out = invoke_in_tx(&foo_sfl, FooIn { age_delta: 11 }).await;
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
