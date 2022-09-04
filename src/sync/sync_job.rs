use super::SyncFeedJob;
use crate::db;
use crate::db::feeds;
use crate::Config;
use fang::typetag;
use fang::FangError;
use fang::Queueable;
use fang::Runnable;
use fang::Scheduled;
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
    fn run(&self, queue: &dyn Queueable) -> Result<(), FangError> {
        let mut connection = crate::db::pool().get()?;

        let mut unsynced_feed_ids: Vec<i64>;
        let mut page = 1;

        log::info!("Started enqueuing feeds for sync");

        let mut total_number = 0;

        let last_synced_at = db::current_time();
        loop {
            unsynced_feed_ids = match feeds::find_unsynced_feeds(
                &mut connection,
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
                queue.insert_task(&SyncFeedJob::new(*id))?;
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

        feeds::increment_and_reset_skips(&mut connection)?;

        Ok(())
    }

    fn cron(&self) -> Option<Scheduled> {
        Some(Scheduled::CronPattern(Config::sync_cron_pattern()))
    }

    fn uniq(&self) -> bool {
        true
    }

    fn task_type(&self) -> String {
        super::JOB_TYPE.to_string()
    }
}
