use super::RemoveOldItemsJob;
use crate::db::feeds;
use crate::Config;
use fang::typetag;
use fang::FangError;
use fang::PgConnection;
use fang::Queueable;
use fang::Runnable;
use fang::Scheduled;
use serde::{Deserialize, Serialize};

const FEEDS_PER_PAGE: i64 = 500;

#[derive(Serialize, Deserialize)]
pub struct CleanJob {}

impl Default for CleanJob {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanJob {
    pub fn new() -> Self {
        CleanJob {}
    }

    pub fn execute(&self, queue: &dyn Queueable) -> Result<(), FangError> {
        let mut conn = crate::db::pool().get()?;

        self.delete_feeds_without_subscriptions(&mut conn);

        let mut current_feed_ids: Vec<i64>;
        let mut page = 1;
        let mut total_number = 0;

        loop {
            current_feed_ids = match feeds::load_feed_ids(&mut conn, page, FEEDS_PER_PAGE) {
                Err(err) => {
                    let description = format!("{:?}", err);
                    return Err(FangError { description });
                }
                Ok(ids) => ids,
            };

            page += 1;

            if current_feed_ids.is_empty() {
                break;
            }

            total_number += current_feed_ids.len();

            for feed_id in current_feed_ids {
                queue.insert_task(&RemoveOldItemsJob::new(feed_id))?;
            }
        }

        log::info!(
            "Finished enqueuing feeds for deletion of old feed items. Total Number:  {}",
            total_number
        );

        Ok(())
    }

    fn delete_feeds_without_subscriptions(&self, conn: &mut PgConnection) {
        log::info!("Started removing feeds without subscriptions");

        match feeds::delete_feeds_without_subscriptions(conn) {
            Ok(count) => log::info!("Removed {} feeds without subscriptions", count),
            Err(error) => log::error!("Failed to remove feeds without subscriptions {:?}", error),
        };
    }
}

#[typetag::serde]
impl Runnable for CleanJob {
    fn run(&self, queue: &dyn Queueable) -> Result<(), FangError> {
        self.execute(queue)
    }

    fn uniq(&self) -> bool {
        true
    }

    fn task_type(&self) -> String {
        super::JOB_TYPE.to_string()
    }

    fn cron(&self) -> Option<Scheduled> {
        Some(Scheduled::CronPattern(Config::clean_cron_pattern()))
    }
}
