use dotenv::dotenv;
use el_monitorro;
use el_monitorro::bot::deliver_job::DeliverJob;
use el_monitorro::db;
use tokio::runtime;
use tokio::time;

fn main() {
    dotenv().ok();
    env_logger::init();

    let mut tokio_runtime = runtime::Builder::new()
        .thread_name("deliver-pool")
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();

    tokio_runtime.block_on(async {
        let mut interval = time::interval(std::time::Duration::from_secs(60));

        loop {
            interval.tick().await;

            if db::semaphore().available_permits() == *db::pool_connection_number() {
                match DeliverJob::new().execute().await {
                    Err(error) => log::error!("Failed to send updates: {}", error.msg),
                    Ok(_) => (),
                }
            } else {
                log::error!("The previous delivery job did not finish");
            }
        }
    })
}
