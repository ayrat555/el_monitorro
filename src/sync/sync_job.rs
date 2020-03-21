use crate::db;
use crate::db::feeds;
use crate::models::feed::Feed;
use crate::sync::feed_sync_job;
use chrono::offset::Utc;
use chrono::DateTime;
use diesel::result::Error;
use izta::job::Job;
use izta::process_jobs;
use izta::runner::Runner;
use izta::task::task_req::TaskReq;
use serde::{Deserialize, Serialize};

pub struct SyncJob {
    last_synced_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncError {
    msg: String,
}

impl SyncJob {
    pub fn new(last_synced_at: DateTime<Utc>) -> Self {
        SyncJob { last_synced_at }
    }

    pub fn execute(&self) -> Result<usize, SyncError> {
        let db_connection = db::establish_connection();

        let mut unsynced_feeds: Vec<Feed>;
        let mut page = 1;

        log::info!("Started enqueuing feeds for sync");

        let mut total_number = 0;

        loop {
            unsynced_feeds =
                feeds::find_unsynced_feeds(&db_connection, self.last_synced_at, page, 100)?;

            page += 1;

            for feed in &unsynced_feeds {
                feed_sync_job::enqueue_job(feed.id);
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

impl Job for SyncJob {
    type R = usize;
    type E = SyncError;

    // All jobs must have a UUID
    const UUID: &'static str = "74f3a15b-75c0-4889-9546-63b02ff304e3";

    const MAX_ATTEMPTS: usize = 3;

    // Job logic - return an `Err` for errors and `Ok` if successful.
    fn run(&self) -> Result<Self::R, Self::E> {
        self.execute()
    }
}
