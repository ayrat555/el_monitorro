use dotenv::dotenv;
use el_monitorro::bot;
use fang::Queue;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let queue = Queue::new();

    el_monitorro::start_scheduler(&queue);

    bot::handler::Handler::start().await;
}
