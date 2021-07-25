use dotenv::dotenv;
use el_monitorro::bot::deliver_job::{DeliverChatUpdatesJob, DeliverJob};
use fang::Queue;
use fang::Runnable;
use fang::WorkerParams;
use fang::WorkerPool;

fn main() {
    dotenv().ok();
    env_logger::init();

    assert_eq!(DeliverJob::new().task_type(), "deliver".to_string());
    assert_eq!(
        DeliverChatUpdatesJob { chat_id: 1 }.task_type(),
        "deliver".to_string()
    );

    Queue::new().remove_tasks_of_type("deliver").unwrap();

    let mut worker_params = WorkerParams::new();
    worker_params.set_task_type("deliver".to_string());
    WorkerPool::new_with_params(10, worker_params).start();

    std::thread::park();
}
