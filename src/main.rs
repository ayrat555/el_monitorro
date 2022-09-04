use dotenv::dotenv;
use el_monitorro::bot;
use el_monitorro::config::Config;
use fang::Queue;

fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let queue = Queue::builder()
        .connection_pool(el_monitorro::db::pool().clone())
        .build();

    if Config::all_binaries() {
        el_monitorro::start_clean_workers(&queue);
        el_monitorro::start_sync_workers(&queue);
        el_monitorro::start_delivery_workers(&queue);
    }

    el_monitorro::start_scheduler(&queue);

    bot::handler::Handler::start();
}
