use crate::bot::deliver_job::{DeliverChatUpdatesJob, DeliverJob};
use crate::cleaner::CleanJob;
use crate::cleaner::RemoveOldItemsJob;
use crate::sync::feed_sync_job::FeedSyncJob;
use crate::sync::sync_job::SyncJob;
use fang::scheduler::Scheduler;
use fang::Queue;
use fang::Runnable;
use fang::WorkerParams;
use fang::WorkerPool;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate log;

pub mod bot;
pub mod cleaner;
pub mod db;
mod models;
mod schema;
pub mod sync;

pub fn start_delivery_workers(queue: &Queue) {
    assert_eq!(DeliverJob::new().task_type(), "deliver".to_string());
    assert_eq!(
        DeliverChatUpdatesJob { chat_id: 1 }.task_type(),
        "deliver".to_string()
    );

    start_workers(queue, "deliver".to_string(), 10);
}

pub fn start_sync_workers(queue: &Queue) {
    assert_eq!(SyncJob::new().task_type(), "sync".to_string());
    assert_eq!(FeedSyncJob::new(1).task_type(), "sync".to_string());

    start_workers(queue, "sync".to_string(), 10);
}

pub fn start_clean_workers(queue: &Queue) {
    assert_eq!(CleanJob::new().task_type(), "clean".to_string());
    assert_eq!(RemoveOldItemsJob::new(1).task_type(), "clean".to_string());

    start_workers(queue, "clean".to_string(), 2);
}

pub fn start_scheduler(queue: &Queue) {
    queue.remove_all_periodic_tasks().unwrap();

    queue.push_periodic_task(&SyncJob::default(), 120).unwrap();

    queue
        .push_periodic_task(&DeliverJob::default(), 60)
        .unwrap();

    queue
        .push_periodic_task(&CleanJob::default(), 12 * 60 * 60)
        .unwrap();

    Scheduler::start(10, 5);
}

fn start_workers(queue: &Queue, typ: String, number: u32) {
    queue.remove_tasks_of_type(&typ).unwrap();

    let mut worker_params = WorkerParams::new();
    worker_params.set_task_type(typ);
    WorkerPool::new_with_params(number, worker_params).start();
}
