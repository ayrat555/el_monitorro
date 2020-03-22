use el_monitorro;
use el_monitorro::sync::{feed_sync_job, sync_job};

fn main() {
    env_logger::init();
    feed_sync_job::start_runner();
    sync_job::start_runner();

    el_monitorro::rocket().launch();
}
