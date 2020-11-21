use dotenv::dotenv;
use el_monitorro;
use el_monitorro::bot::deliver_job::DeliverJob;
use tokio::runtime;
use tokio::time;

fn main() {
    dotenv().ok();
    env_logger::init();

    let mut tokio_runtime = runtime::Builder::new()
        .thread_name("deliver-pool")
        .basic_scheduler()
        .core_threads(1)
        .max_threads(1)
        .on_thread_start(|| {
            println!("thread started");
        })
        .enable_time()
        .build()
        .unwrap();

    tokio_runtime.block_on(async {
        let mut interval = time::interval(std::time::Duration::from_secs(60));

        loop {
            match DeliverJob::new().execute() {
                Err(error) => log::error!("Failed to send updates: {}", error.msg),
                Ok(_) => (),
            }

            interval.tick().await;
        }
    })
}
