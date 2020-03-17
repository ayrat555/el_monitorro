use el_monitorro;
use el_monitorro::sync::runner::{enqueue_job, start_runners};

fn main() {
    env_logger::init();
    start_runners(1);
    enqueue_job(1);

    el_monitorro::rocket().launch();
}
