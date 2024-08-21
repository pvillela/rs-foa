use dev_support::artct::tokio_run_artct::{run, RunIn};

#[tokio::main]
async fn main() {
    println!("===== cda_run_foo_artct_bar_artct_tokio =====");

    run(RunIn {
        unit_time_millis: 1,
        app_cfg_first_refresh_units: 10,
        app_cfg_refresh_delta_units: 10,
        app_cfg_refresh_count: 10,
        per_call_sleep_units: 1,
        increment_to_print: 33,
        concurrency: 1_000,
        repeats: 100,
    })
    .await;
}
