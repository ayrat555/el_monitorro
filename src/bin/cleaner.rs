use dotenv::dotenv;
use el_monitorro::cleaner::clean_job::CleanJob;
use el_monitorro::cleaner::clean_job::RemoveOldItemsJob;
use fang::Queue;
use fang::Runnable;
use fang::WorkerParams;
use fang::WorkerPool;

fn main() {
    dotenv().ok();

    assert_eq!(CleanJob::new().task_type(), "clean".to_string());
    assert_eq!(RemoveOldItemsJob::new(1).task_type(), "clean".to_string());

    Queue::new().remove_tasks_of_type("clean").unwrap();

    let mut worker_params = WorkerParams::new();
    worker_params.set_task_type("clean".to_string());
    WorkerPool::new_with_params(2, worker_params).start();

    std::thread::park();
}
