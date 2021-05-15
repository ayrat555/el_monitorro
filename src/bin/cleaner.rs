use dotenv::dotenv;

use el_monitorro::cleaner::clean_job::CleanJob;
use el_monitorro::db;
use std::env;
use tokio::runtime;
use tokio::time;

fn main() {
    dotenv().ok();
    env_logger::init();

    let tokio_runtime = runtime::Builder::new_multi_thread()
        .thread_name("clean-pool")
        .enable_all()
        .build()
        .unwrap();

    let period: u64 = env::var("CLEAN_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .unwrap();

    tokio_runtime.block_on(async {
        let mut interval = time::interval(std::time::Duration::from_secs(period));

        loop {
            interval.tick().await;

            if db::semaphore().available_permits() == *db::pool_connection_number() {
                if let Err(error) = CleanJob::new().execute().await {
                    log::error!("Failed to remove old feed items: {}", error.msg)
                }
            } else {
                log::error!("The previous clean job did not finish");
            }
        }
    })
}
