use dotenv::dotenv;
use fang::Queue;

fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let queue = Queue::builder()
        .connection_pool(el_monitorro::db::pool().clone())
        .build();

    el_monitorro::start_clean_workers(&queue);

    std::thread::park();
}
