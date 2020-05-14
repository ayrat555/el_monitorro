use crate::db;
use crate::db::feeds;
use crate::sync::feed_sync_job::FeedSyncJob;

use diesel::result::Error;
use tokio::time;

pub struct SyncJob {}

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

        let mut unsynced_feed_ids: Vec<i64>;
        let mut page = 1;

        log::info!("Started enqueuing feeds for sync");

        let mut total_number = 0;

        let last_synced_at = db::current_time();
        loop {
            unsynced_feed_ids =
                feeds::find_unsynced_feeds(&db_connection, last_synced_at, page, 100)?;

            page += 1;

            for id in &unsynced_feed_ids {
                tokio::spawn(sync_feed(*id));
            }

            if unsynced_feed_ids.is_empty() {
                break;
            }

            total_number += unsynced_feed_ids.len();
        }

        log::info!(
            "Finished enqueuing feeds for sync. Total Number:  {}",
            total_number
        );

        Ok(total_number)
    }
}

pub fn sync_all_feeds() {
    match SyncJob::new().execute() {
        Err(error) => log::error!("Failed to sync feeds: {}", error.msg),
        Ok(_) => (),
    }
}

pub async fn sync_feeds() {
    let mut interval = time::interval(std::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        sync_all_feeds();
    }
}

pub async fn sync_feed(feed_id: i64) {
    match FeedSyncJob::new(feed_id).execute() {
        Err(_error) => log::error!("Failed to process feed {}", feed_id),
        Ok(_) => (),
    }
}
