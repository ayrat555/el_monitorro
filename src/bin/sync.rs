use dotenv::dotenv;
use el_monitorro::sync::sync_job::{SyncFeedJob, SyncJob};
use fang::Runnable;
use fang::WorkerParams;
use fang::WorkerPool;

fn main() {
    dotenv().ok();
    env_logger::init();

    assert_eq!(SyncJob::new().task_type(), "sync".to_string());
    assert_eq!(SyncFeedJob { feed_id: 1 }.task_type(), "sync".to_string());

    let mut worker_params = WorkerParams::new();
    worker_params.set_task_type("sync".to_string());

    WorkerPool::new_with_params(10, worker_params).start();

    std::thread::park();
}
