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

const SCHEDULER_CHECK_PERIOD: u32 = 10;
const SCHEDULER_ERROR_MARGIN_SECONDS: u32 = 10;

pub fn fix_units_for_cron(seconds_amount: u32) -> Vec<u32> {
    let mut vec = vec![];
    let mut unit = seconds_amount;
    for div in [60, 60, 24] {
        if unit < div {
            vec.push(unit);
            break;
        } else {
            vec.push(unit % div);

            unit = unit / div;
        }
    }
    if vec.len() == 3 {
        vec.push(unit);
    }
    vec
}

pub fn seconds_to_pattern(seconds_amount: u32) -> String {
    let vec = fix_units_for_cron(seconds_amount);

    match vec.len() {
        1 => format!("*/{} * * * * * *", vec[0]),
        2 => format!("*/{} */{} * * * * *", vec[0], vec[1]),
        3 => format!("*/{} */{} */{} * * * *", vec[0], vec[1], vec[2]),
        4 => format!("*/{} */{} */{} */{} * * *", vec[0], vec[1], vec[2], vec[3]),
        _ => panic!("Error fix units for cron"),
    }
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
