#![allow(clippy::extra_unused_lifetimes)]

use crate::cleaner::CleanJob;
use crate::config::Config;
use crate::deliver::DeliverJob;
use crate::sync::SyncJob;
use fang::Queue;
use fang::Queueable;
use fang::RetentionMode;
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
mod schema;
pub mod sync;

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
    queue.remove_all_tasks().unwrap();

    queue.schedule_task(&SyncJob::default()).unwrap();

    queue.schedule_task(&DeliverJob::default()).unwrap();

    queue.schedule_task(&CleanJob::default()).unwrap();
}

fn start_workers(queue: &Queue, typ: String, number: u32) {
    queue.remove_tasks_of_type(&typ).unwrap();

    let mut worker_pool = WorkerPool::<Queue>::builder()
        .queue(queue.clone())
        .retention_mode(RetentionMode::RemoveAll)
        .number_of_workers(number)
        .task_type(typ)
        .build();

    worker_pool.start().unwrap()
}
