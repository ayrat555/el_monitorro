use dotenv::dotenv;
use el_monitorro::sync::feed_sync_job::FeedSyncJob;
use el_monitorro::sync::sync_job::SyncJob;
use fang::Queue;
use fang::Runnable;
use fang::WorkerParams;
use fang::WorkerPool;

fn main() {
    dotenv().ok();

    assert_eq!(SyncJob::new().task_type(), "sync".to_string());
    assert_eq!(FeedSyncJob::new(1).task_type(), "sync".to_string());

    Queue::new().remove_tasks_of_type("sync").unwrap();

    let mut worker_params = WorkerParams::new();
    worker_params.set_task_type("sync".to_string());

    WorkerPool::new_with_params(10, worker_params).start();

    std::thread::park();
}
