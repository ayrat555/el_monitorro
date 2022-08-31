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

const SCHEDULER_CHECK_PERIOD: u64 = 10;
const SCHEDULER_ERROR_MARGIN_SECONDS: u64 = 10;

pub fn fix_units_for_cron(seconds_amount: u64) -> Vec<u64> {
    let (seconds, minutes): (u64, u64) = if seconds_amount > 59 {
        (seconds_amount % 60, seconds_amount / 60)
    } else {
        return vec![seconds_amount];
    };

    let (minutes, hours): (u64, u64) = if minutes > 59 {
        (minutes % 60, minutes / 60)
    } else {
        return vec![seconds, minutes];
    };

    let (hours, days): (u64, u64) = if hours > 23 {
        (hours % 24, hours / 24)
    } else {
        return vec![seconds, minutes, hours];
    };

    vec![seconds, minutes, hours, days]
}

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
    queue.remove_all_scheduled_tasks().unwrap();

    // DO CRON METHODS AND FINISH THIS.

    queue
        .schedule_task(
            &SyncJob::default(),
            (Config::sync_interval_in_seconds() * 1_000) as i64,
        )
        .unwrap();

    queue
        .schedule_task(
            &DeliverJob::default(),
            (Config::deliver_interval_in_seconds() * 1_000) as i64,
        )
        .unwrap();

    queue
        .schedule_task(
            &CleanJob::default(),
            (Config::clean_interval_in_seconds() * 1_000) as i64,
        )
        .unwrap();
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
