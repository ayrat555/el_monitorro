use dotenv::dotenv;
use el_monitorro::bot;
use fang::Queue;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let queue = Queue::new();

    el_monitorro::start_scheduler(&queue);
    el_monitorro::start_sync_workers(&queue);
    el_monitorro::start_delivery_workers(&queue);
    el_monitorro::start_clean_workers(&queue);

    bot::handler::Handler::start().await;
}
