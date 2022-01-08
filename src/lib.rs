use crate::cleaner::CleanJob;
use crate::config::Config;
use crate::deliver::DeliverJob;
use crate::sync::SyncJob;
use fang::scheduler::Scheduler;
use fang::Queue;
use fang::RetentionMode;
use fang::WorkerParams;
use fang::WorkerPool;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate log;

pub mod bot;
pub mod cleaner;
pub mod config;
pub mod db;
pub mod deliver;
mod models;
pub mod populate_content_hash;
mod schema;
pub mod sync;

const SCHEDULER_CHECK_PERIOD: u64 = 10;
const SCHEDULER_ERROR_MARGIN_SECONDS: u64 = 10;

pub fn start_delivery_workers(queue: &Queue) {
    start_workers(
        queue,
        "deliver".to_string(),
        Config::deliver_workers_number(),
    );
}

pub fn start_sync_workers(queue: &Queue) {
    start_workers(queue, "sync".to_string(), Config::sync_workers_number());
}

pub fn start_clean_workers(queue: &Queue) {
    start_workers(queue, "clean".to_string(), Config::clean_workers_number());
}

pub fn start_scheduler(queue: &Queue) {
    queue.remove_all_periodic_tasks().unwrap();

    queue
        .push_periodic_task(&SyncJob::default(), Config::sync_interval_in_seconds())
        .unwrap();

    queue
        .push_periodic_task(
            &DeliverJob::default(),
            Config::deliver_interval_in_seconds(),
        )
        .unwrap();

    queue
        .push_periodic_task(&CleanJob::default(), Config::clean_interval_in_seconds())
        .unwrap();

    Scheduler::start(SCHEDULER_CHECK_PERIOD, SCHEDULER_ERROR_MARGIN_SECONDS);
}

fn start_workers(queue: &Queue, typ: String, number: u32) {
    queue.remove_tasks_of_type(&typ).unwrap();

    let mut worker_params = WorkerParams::new();
    worker_params.set_task_type(typ);
    worker_params.set_retention_mode(RetentionMode::RemoveAll);
    WorkerPool::new_with_params(number, worker_params)
        .start()
        .unwrap();
}
