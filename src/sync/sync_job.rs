use crate::db;
use crate::db::feeds;
use crate::models::feed::Feed;
use crate::sync::feed_sync_job::{FeedSyncError, FeedSyncJob};
use chrono::Duration;
use diesel::result::Error;
use dotenv::dotenv;
use izta::job::Job;
use izta::process_jobs;
use izta::runner::Runner;
use izta::task::task_req::TaskReq;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct SyncJob {}

#[derive(Serialize, Deserialize)]
pub struct SyncError {
    msg: String,
}

impl From<Error> for SyncError {
    fn from(error: Error) -> Self {
        let msg = format!("{:?}", error);

        SyncError { msg }
    }
}

impl SyncJob {
    pub fn new() -> Self {
        SyncJob {}
    }

    pub fn execute(&self) -> Result<usize, SyncError> {
        let db_connection = db::establish_connection();

        let mut unsynced_feeds: Vec<Feed>;
        let mut page = 1;

        log::info!("Started enqueuing feeds for sync");

        let mut total_number = 0;

        let last_synced_at = db::current_time() - Duration::hours(24);
        loop {
            unsynced_feeds = feeds::find_unsynced_feeds(&db_connection, last_synced_at, page, 100)?;

            page += 1;

            for feed in &unsynced_feeds {
                enqueue_job(feed.id);
            }

            if unsynced_feeds == [] {
                break;
            }

            total_number += unsynced_feeds.len();
        }

        log::info!(
            "Finished enqueuing feeds for sync. Total Number:  {}",
            total_number
        );

        Ok(total_number)
    }
}

pub fn enqueue_job(number: i32) {
    dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found");
    let runner = Runner::new(
        process_jobs!(FeedSyncJob),
        &database_url,
        "tasks",
        vec!["feed_sync_jobs".to_string()],
    );

    let task_req = TaskReq::new(FeedSyncJob::new(number));
    runner.add_task(&task_req);
}

impl Job for FeedSyncJob {
    type R = ();
    type E = FeedSyncError;

    // All jobs must have a UUID
    const UUID: &'static str = "74f3a15b-75c0-4889-9546-63b02ff304e4";

    const MAX_ATTEMPTS: usize = 3;

    // Job logic - return an `Err` for errors and `Ok` if successful.
    fn run(&self) -> Result<Self::R, Self::E> {
        self.execute()
    }
}

pub fn start_runner() {
    dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found");
    let runner = Runner::new(
        process_jobs!(FeedSyncJob, SyncJob),
        &database_url,
        "tasks",
        vec![],
    );

    runner.start();

    let task_req = TaskReq::new(SyncJob::new());
    let periodic_req = TaskReq::run_every(task_req, Duration::hours(1));
    runner.add_task(&periodic_req);
}

impl Job for SyncJob {
    type R = usize;
    type E = SyncError;

    // All jobs must have a UUID
    const UUID: &'static str = "74f3a15b-75c0-4889-9546-63b02ff304e2";

    const MAX_ATTEMPTS: usize = 3;

    // Job logic - return an `Err` for errors and `Ok` if successful.
    fn run(&self) -> Result<Self::R, Self::E> {
        self.execute()
    }
}
