use dotenv::dotenv;
use el_monitorro;
use el_monitorro::sync::sync_job;
use tokio::runtime;
use tokio::time;

fn main() {
    dotenv().ok();
    env_logger::init();

    let mut tokio_runtime = runtime::Builder::new()
        .threaded_scheduler()
        .core_threads(2)
        .max_threads(3)
        .enable_time()
        .build()
        .unwrap();

    tokio_runtime.block_on(async {
        let mut interval = time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            sync_job::sync_all_feeds();
        }
    })
}
