use dotenv::dotenv;
use el_monitorro::sync::*;
use fang::typetag;
use fang::WorkerParams;
use fang::WorkerPool;
use fang::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Bar {}

#[typetag::serde]
impl fang::Runnable for Bar {
    fn run(&self) -> Result<(), fang::Error> {
        Ok(())
    }
}

fn main() {
    dotenv().ok();
    env_logger::init();

    let mut worker_params = WorkerParams::new();
    worker_params.set_task_type("sync".to_string());

    WorkerPool::new_with_params(10, worker_params).start();

    std::thread::park();
}
