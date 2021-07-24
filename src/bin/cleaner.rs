use dotenv::dotenv;
use el_monitorro::cleaner::clean_job::CleanJob;
use el_monitorro::cleaner::clean_job::RemoveOldItemsJob;
use fang::Runnable;
use fang::WorkerParams;
use fang::WorkerPool;

fn main() {
    dotenv().ok();
    env_logger::init();

    assert_eq!(CleanJob::new().task_type(), "clean".to_string());
    assert_eq!(RemoveOldItemsJob::new(1).task_type(), "clean".to_string());

    let mut worker_params = WorkerParams::new();
    worker_params.set_task_type("clean".to_string());
    WorkerPool::new_with_params(10, worker_params).start();

    std::thread::park();
}
