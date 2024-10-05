use app1::run::{
    ctx::Ctx,
    tokio_run::{run, RunIn},
};

#[tokio::main]
async fn main() {
    println!("===== app1-cmd =====");

    Ctx::init().await; // initialize context

    run(RunIn {
        unit_time_millis: 1,
        app_cfg_first_refresh_units: 10,
        app_cfg_refresh_delta_units: 10,
        app_cfg_refresh_count: 10,
        increment_to_print: 10,
        concurrency: 5,
        repeats: 100,
    })
    .await;
}
