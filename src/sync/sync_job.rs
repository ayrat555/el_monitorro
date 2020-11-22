use crate::bot::api;
use crate::db;
use crate::db::feeds;
use crate::db::telegram;
use crate::sync::feed_sync_job::{FeedSyncError, FeedSyncJob};
use diesel::result::Error;

pub struct SyncJob {}

pub struct SyncError {
    pub msg: String,
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

    pub async fn execute(&self) -> Result<usize, SyncError> {
        let semaphored_connection = db::get_semaphored_connection().await;
        let db_connection = semaphored_connection.connection;

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

pub async fn sync_feed(feed_id: i64) {
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;
    let feed_sync_result = FeedSyncJob::new(feed_id).execute(&db_connection);

    match feed_sync_result {
        Err(FeedSyncError::StaleError) => {
            log::error!("Feed can not be processed for a long time {}", feed_id);

            let feed = feeds::find(&db_connection, feed_id).unwrap();
            let chats = telegram::find_chats_by_feed_id(&db_connection, feed_id).unwrap();

            let message = format!("{} can not be processed. It was removed.", feed.link);

            for chat in chats.into_iter() {
                match api::send_message(chat.id, message.clone()).await {
                    Ok(_) => (),
                    Err(error) => {
                        log::error!("Failed to send a message: {}", error);
                    }
                }
            }

            match feeds::remove_feed(&db_connection, feed_id) {
                Ok(_) => log::info!("Feed was removed: {}", feed_id),
                Err(err) => log::error!("Failed to remove feed: {} {}", feed_id, err),
            }
        }
        Err(error) => log::error!("Failed to process feed {}: {:?}", feed_id, error),
        Ok(_) => (),
    }
}
