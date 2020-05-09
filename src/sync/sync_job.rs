use crate::db;
use crate::db::feeds;
use crate::models::feed::Feed;
use crate::sync::feed_sync_job::{FeedSyncError, FeedSyncJob};
use chrono::Duration;
use diesel::result::Error;
// use dotenv::dotenv;
// use serde::{Deserialize, Serialize};
// use std::env;

// #[derive(Serialize, Deserialize)]
pub struct SyncJob {}

// #[derive(Serialize, Deserialize)]
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
                tokio::spawn(sync_feed(feed.id));
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

pub async fn sync_all_feeds() {
    match SyncJob::new().execute() {
        Err(_error) => log::error!("Failed to sync feeds"),
        Ok(_) => (),
    }
}

pub async fn sync_feed(feed_id: i64) {
    match FeedSyncJob::new(feed_id).execute() {
        Err(_error) => log::error!("Failed to process feed {}", feed_id),
        Ok(_) => (),
    }
}
