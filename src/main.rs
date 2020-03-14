use el_monitorro;
use el_monitorro::sync::queue;

fn main() {
    queue::job_queue();
    env_logger::init();
    el_monitorro::rocket().launch();
}
