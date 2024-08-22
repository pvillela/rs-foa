use dev_support::artctps::tokio_run::{run, RunIn};

#[tokio::main]
async fn main() {
    println!("===== cda_run_foo_bar_artctps_tokio =====");

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
