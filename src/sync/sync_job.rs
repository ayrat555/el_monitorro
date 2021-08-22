use super::SyncFeedJob;
use crate::db;
use crate::db::feeds;
use fang::typetag;
use fang::Error as FangError;
use fang::PgConnection;
use fang::Queue;
use fang::Runnable;
use serde::{Deserialize, Serialize};

const FEEDS_PER_PAGE: i64 = 100;

#[derive(Serialize, Deserialize)]
pub struct SyncJob {}

impl Default for SyncJob {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SyncError {
    pub msg: String,
}

impl SyncJob {
    pub fn new() -> Self {
        SyncJob {}
    }
}

#[typetag::serde]
impl Runnable for SyncJob {
    fn run(&self, connection: &PgConnection) -> Result<(), FangError> {
        let mut unsynced_feed_ids: Vec<i64>;
        let mut page = 1;

        log::info!("Started enqueuing feeds for sync");

        let mut total_number = 0;

        let last_synced_at = db::current_time();
        loop {
            unsynced_feed_ids = match feeds::find_unsynced_feeds(
                connection,
                last_synced_at,
                page,
                FEEDS_PER_PAGE,
            ) {
                Ok(ids) => ids,
                Err(err) => {
                    let description = format!("{:?}", err);

                    return Err(FangError { description });
                }
            };

            page += 1;

            for id in &unsynced_feed_ids {
                Queue::push_task_query(connection, &SyncFeedJob::new(*id)).unwrap();
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

        Ok(())
    }

    fn task_type(&self) -> String {
        super::JOB_TYPE.to_string()
    }
}
