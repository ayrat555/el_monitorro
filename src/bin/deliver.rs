use dotenv::dotenv;

use el_monitorro::bot::deliver_job::DeliverJob;
use el_monitorro::db;
use fang::WorkerParams;
use fang::WorkerPool;
use std::env;
use tokio::runtime;
use tokio::time;

fn main() {
    dotenv().ok();
    env_logger::init();

    let mut worker_params = WorkerParams::new();
    worker_params.set_task_type("deliver".to_string());
    WorkerPool::new_with_params(10, worker_params).start();

    let tokio_runtime = runtime::Builder::new_multi_thread()
        .thread_name("deliver-pool")
        .enable_all()
        .build()
        .unwrap();

    let period: u64 = env::var("DELIVER_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap();

    tokio_runtime.block_on(async {
        let mut interval = time::interval(std::time::Duration::from_secs(period));

        loop {
            interval.tick().await;

            if db::semaphore().available_permits() == *db::pool_connection_number() {
                if let Err(error) = DeliverJob::new().execute().await {
                    log::error!("Failed to send updates: {}", error.msg)
                }
            } else {
                log::error!("The previous delivery job did not finish");
            }
        }
    })
}
