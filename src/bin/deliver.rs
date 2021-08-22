use dotenv::dotenv;
use fang::Queue;

fn main() {
    dotenv().ok();
    env_logger::init();

    let queue = Queue::new();

    el_monitorro::start_delivery_workers(&queue);

    std::thread::park();
}
