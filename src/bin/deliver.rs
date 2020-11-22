use dotenv::dotenv;
use el_monitorro;
use el_monitorro::bot::deliver_job;
use el_monitorro::bot::deliver_job::DeliverJob;
use tokio::runtime;
use tokio::time;

fn main() {
    dotenv().ok();
    env_logger::init();

    let mut tokio_runtime = runtime::Builder::new()
        .thread_name("deliver-pool")
        .threaded_scheduler()
        .on_thread_start(|| {
            println!("thread started");
        })
        .enable_all()
        .build()
        .unwrap();

    tokio_runtime.block_on(async {
        let mut interval = time::interval(std::time::Duration::from_secs(60));

        loop {
            interval.tick().await;

            match DeliverJob::new().execute().await {
                Err(error) => log::error!("Failed to send updates: {}", error.msg),
                Ok(_) => (),
            }
        }
    })
}
