use el_monitorro;
use el_monitorro::bot;
use el_monitorro::sync::sync_job;

fn main() {
    env_logger::init();
    sync_job::start_runner();
    bot::api::start_bot();

    el_monitorro::rocket().launch();
}
