use crate::bot::api;
use crate::db;
use crate::db::feeds;
use crate::db::telegram;
use crate::sync::feed_sync_job::{FeedSyncError, FeedSyncJob};
use diesel::result::Error;
use fang::Error as FangError;
use fang::Postgres;
use fang::Runnable;
use serde::{Deserialize, Serialize};

pub struct SyncJob {}

impl Default for SyncJob {
    fn default() -> Self {
        Self::new()
    }
}

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
        let postgres = Postgres::new(None);

        let mut unsynced_feed_ids: Vec<i64>;
        let mut page = 1;

        log::info!("Started enqueuing feeds for sync");

        let mut total_number = 0;

        let last_synced_at = db::current_time();
        loop {
            unsynced_feed_ids =
                feeds::find_unsynced_feeds(&postgres.connection, last_synced_at, page, 100)?;

            page += 1;

            for id in &unsynced_feed_ids {
                postgres.push_task(&SyncFeedJob { feed_id: *id }).unwrap();
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

#[derive(Serialize, Deserialize)]
pub struct SyncFeedJob {
    pub feed_id: i64,
}

impl SyncFeedJob {
    pub fn sync_feed(&self) {
        let db_connection = db::establish_connection();
        let feed_sync_result = FeedSyncJob::new(self.feed_id).execute(&db_connection);
        let feed_id = self.feed_id;

        match feed_sync_result {
            Err(FeedSyncError::StaleError) => {
                log::error!("Feed can not be processed for a long time {}", feed_id);

                let feed = feeds::find(&db_connection, feed_id).unwrap();
                let chats = telegram::find_chats_by_feed_id(&db_connection, feed_id).unwrap();

                let message = format!("{} can not be processed. It was removed.", feed.link);

                for chat in chats.into_iter() {
                    match api::send_message_sync(chat.id, message.clone()) {
                        Ok(_) => (),
                        Err(error) => {
                            log::error!("Failed to send a message: {:?}", error);
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
}

#[typetag::serde]
impl Runnable for SyncFeedJob {
    fn run(&self) -> Result<(), FangError> {
        self.sync_feed();

        Ok(())
    }

    fn task_type(&self) -> String {
        "sync".to_string()
    }
}
