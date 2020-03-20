use el_monitorro;
use el_monitorro::sync::feed_sync_job;

fn main() {
    env_logger::init();
    feed_sync_job::start_runners(1);

    el_monitorro::rocket().launch();
}
