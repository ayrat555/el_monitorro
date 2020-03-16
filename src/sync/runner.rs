use crate::sync::job::SyncError;
use crate::sync::job::SyncJob;
use dotenv::dotenv;
use izta::job::Job;
use izta::process_jobs;
use izta::runner::Runner;
use izta::task::task_req::TaskReq;

use std::env;

pub fn start_runners(number: i32) {
    dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found");
    let runner = Runner::new(process_jobs!(SyncJob), &database_url, "tasks", vec![]);

    for _ in 0..number {
        runner.start();
    }
}

pub fn enqueue_job(number: i32) {
    let database_url =
        env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found");
    let runner = Runner::new(process_jobs!(SyncJob), &database_url, "tasks", vec![]);

    let task_req = TaskReq::new(SyncJob::new(number));
    runner.add_task(&task_req);
}

impl Job for SyncJob {
    type R = ();
    type E = SyncError;

    // All jobs must have a UUID
    const UUID: &'static str = "74f3a15b-75c0-4889-9546-63b02ff304e4";

    const MAX_ATTEMPTS: usize = 3;

    // Job logic - return an `Err` for errors and `Ok` if successful.
    fn run(&self) -> Result<Self::R, Self::E> {
        self.execute()
    }
}
