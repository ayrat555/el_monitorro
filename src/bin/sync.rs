use dotenv::dotenv;

use el_monitorro::db;
use el_monitorro::sync::sync_job::SyncJob;
use std::env;
use tokio::runtime;
use tokio::time;

fn main() {
    dotenv().ok();
    env_logger::init();

    let tokio_runtime = runtime::Builder::new_multi_thread()
        .thread_name("sync-pool")
        .enable_all()
        .build()
        .unwrap();

    let period: u64 = env::var("SYNC_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap();

    tokio_runtime.block_on(async {
        let mut interval = time::interval(std::time::Duration::from_secs(period));

        loop {
            interval.tick().await;

            if db::semaphore().available_permits() == *db::pool_connection_number() {
                if let Err(error) = SyncJob::new().execute().await {
                    log::error!("Failed to sync feeds: {}", error.msg)
                }
            } else {
                log::error!("The previous sync job did not finish");
            }
        }
    })
}
