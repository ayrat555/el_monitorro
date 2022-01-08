use dotenv::dotenv;
use el_monitorro::bot;
use el_monitorro::config::Config;
use el_monitorro::populate_content_hash::PopulateContentHashJob;
use fang::Queue;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let queue = Queue::new();

    queue.push_task(&PopulateContentHashJob::default()).unwrap();

    if Config::all_binaries() {
        el_monitorro::start_clean_workers(&queue);
        el_monitorro::start_sync_workers(&queue);
        el_monitorro::start_delivery_workers(&queue);
    }

    el_monitorro::start_scheduler(&queue);

    bot::handler::Handler::start().await;
}
